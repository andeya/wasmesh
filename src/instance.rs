use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

use wasmer::{Function, FunctionType, import_namespace, ImportObject, Memory, MemoryView, Module, Store};
#[cfg(not(feature = "llvm"))]
use wasmer_compiler_cranelift::Cranelift;
#[cfg(feature = "llvm")]
use wasmer_compiler_llvm::LLVM;
use wasmer_engine_universal::Universal;
use wasmer_wasi::{WasiEnv, WasiState};

use crate::server::ServeOpt;

#[derive(Clone, Debug)]
pub(crate) struct Instance {
    instance: wasmer::Instance,
    message_cache: RefCell<HashMap<i32, Vec<u8>>>,
    ctx_id_count: RefCell<i32>,
}

static mut INSTANCES: Vec<Instance> = Vec::new();

fn instance_ref(index: usize) -> &'static Instance {
    let real_index = index % unsafe { INSTANCES.len() };
    #[cfg(debug_assertions)]
    println!("index: {}%{}={}", index, unsafe { INSTANCES.len() }, real_index);
    return unsafe { &INSTANCES[real_index] }
}

pub(crate) fn local_instance_ref() -> (usize, &'static Instance) {
    let thread_id = current_thread_id();
    (thread_id, instance_ref(thread_id))
}

pub(crate) async fn rebuild(serve_options: &ServeOpt) -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let (wasi_env, module, store) = Instance::compile(serve_options)?;
        let mut hdls = vec![];
        for _ in 0..serve_options.get_worker_threads() {
            let (_wasi_env, _module, _store) = (wasi_env.clone(), module.clone(), store.clone());
            let hdl = tokio::task::spawn_blocking(move || {
                Instance::new(_wasi_env, _module, _store)
                    .or_else(|e| {
                        eprintln!("{}", e);
                        Err(e)
                    })
                    .unwrap()
            });
            hdls.push(hdl);
        }
        for hdl in hdls {
            INSTANCES.push(hdl.await?);
        }
    }
    Ok(())
}

impl Instance {
    fn compile(serve_options: &ServeOpt) -> Result<(WasiEnv, Module, Store), Box<dyn std::error::Error>> {
        let file_ref: &Path = serve_options.get_wasm_path().as_ref();
        let canonical = file_ref.canonicalize()?;
        let wasm_bytes = std::fs::read(file_ref)?;
        let filename = canonical.as_path().to_str().unwrap();

        // Create a Store.
        // Note that we don't need to specify the engine/compiler if we want to use
        // the default provided by Wasmer.
        // You can use `Store::default()` for that.

        let store: Store;
        #[cfg(not(feature = "llvm"))] {
            store = Store::new(&Universal::new(Cranelift::default()).engine());
        }
        #[cfg(feature = "llvm")] {
            store = Store::new(&Universal::new(LLVM::default()).engine());
        }

        println!("Compiling module {}...", filename);

        let mut module = Module::new(&store, wasm_bytes)?;
        module.set_name(filename);

        println!("Module exports functions: {:?}", module.exports().functions().collect::<Vec<wasmer::ExportType<FunctionType>>>());

        // First, we create the `WasiEnv` with the stdio pipes
        // let input = Pipe::new();
        // let output = Pipe::new();
        let wasi_env = WasiState::new(serve_options.get_name())
            .preopen_dirs(serve_options.get_preopen_dirs())?
            .args(serve_options.to_args_unchecked())
            // .stdin(Box::new(input))
            // .stdout(Box::new(output))
            .finalize()?;
        Ok((wasi_env, module, store))
    }
    fn new(mut wasi_env: WasiEnv, module: Module, store: Store) -> Result<Instance, Box<dyn std::error::Error>> {
        let thread_id = ::std::thread::current().id();

        println!("[{:?}] Instantiating module with WASI imports...", thread_id);

        // Then, we get the import object related to our WASI
        // and attach it to the Wasm instance.
        let mut import_object = wasi_env.import_object(&module)?;
        Self::register_import_object(&mut import_object, &store);

        let instance = wasmer::Instance::new(&module, &import_object)?;

        println!("[{:?}] Created instance: {:?}", thread_id, module.name().unwrap());

        Ok(Instance {
            instance,
            message_cache: RefCell::new(HashMap::with_capacity(1024)),
            ctx_id_count: RefCell::new(0),
        }.init())
    }
    fn register_import_object(import_object: &mut ImportObject, store: &Store) {
        import_object.register("env", import_namespace!({
            "_wasp_host_recall_msg" => Function::new_native(store, |thread_id: i32, ctx_id: i32, offset: i32| {
                // println!("_wasp_host_recall_msg: thread_id:{}, ctx_id:{}, offset:{}", thread_id, ctx_id, offset);
                let ins = instance_ref(thread_id as usize);
                ins.take_msg_data(ctx_id).map(|data|{
                    ins.set_view_bytes(offset as usize, data.iter());
                });
            }),
            "_wasp_host_reply_msg" => Function::new_native(store, |thread_id: i32, ctx_id: i32, offset: i32, size: i32| {
                // println!("_wasp_host_reply_msg: thread_id:{}, ctx_id:{}, offset:{}", thread_id, ctx_id, offset);
                let ins = instance_ref(thread_id as usize);
                let data = ins.get_view_bytes(offset as usize, size as usize);
                ins.cache_msg_data(ctx_id, data);
            }),
            "_wasp_host_send_msg" => Function::new_native(store, |thread_id: i32, ctx_id: i32, offset: i32, size: i32|-> i32 {
                println!("_wasp_host_reply_msg: thread_id:{}, ctx_id:{}, offset:{}, size:{}", thread_id, ctx_id, offset, size);
                // TODO
                0
            }),
        }));
    }
    fn init(self) -> Self {
        self
    }
    fn take_msg_data(&self, ctx_id: i32) -> Option<Vec<u8>> {
        self.message_cache.borrow_mut().remove(&ctx_id)
    }
    fn cache_msg_data(&self, ctx_id: i32, data: Vec<u8>) {
        self.message_cache.borrow_mut().insert(ctx_id, data);
    }
    pub(crate) fn call_guest_handler(&self, thread_id: i32, ctx_id: i32, size: i32) {
        loop {
            if let Err(e) = self
                .instance
                .exports
                .get_native_function::<(i32, i32, i32), ()>("_wasp_guest_handler")
                .unwrap()
                .call(thread_id, ctx_id, size)
            {
                let estr = format!("{:?}", e);
                eprintln!("call _wasp_guest_handler error: {}", estr);
                if estr.contains("OOM") {
                    match self.get_memory().grow(1) {
                        Ok(p) => {
                            println!("memory grow, previous memory size: {:?}", p);
                        },
                        Err(e) => {
                            eprintln!("failed to memory grow: {:?}", e);
                        }
                    }
                }
            } else {
                return
            }
        }
    }
    fn get_memory(&self) -> &Memory {
        self.instance.exports.get_memory("memory").unwrap()
    }
    fn get_view(&self) -> MemoryView<u8> {
        self.get_memory().view::<u8>()
    }
    fn set_view_bytes<'a>(&self, offset: usize, data: impl IntoIterator<Item=&'a u8> + ExactSizeIterator) {
        let view = self.get_view();
        for (cell, b) in view[offset..offset + data.len()].iter().zip(data) {
            cell.set(*b);
        }
    }
    fn get_view_bytes(&self, offset: usize, size: usize) -> Vec<u8> {
        // println!("get_view_bytes: offset:{}, size:{}", offset, size);
        let view = self.get_view();
        view[offset..(offset + size)]
            .iter()
            .map(|c| c.get())
            .collect()
    }
    pub(crate) fn gen_ctx_id(&self) -> i32 {
        self.ctx_id_count.replace_with(|v| *v + 1)
    }
    pub(crate) fn set_guest_request(&self, ctx_id: i32, data: Vec<u8>) -> i32 {
        let size = data.len() as i32;
        self.cache_msg_data(ctx_id, data);
        size
    }
    pub(crate) fn get_guest_response(&self, ctx_id: i32) -> Vec<u8> {
        self.take_msg_data(ctx_id).unwrap_or(vec![])
    }
}

fn current_thread_id() -> usize {
    let thread_id: usize = format!("{:?}", ::std::thread::current().id())
        .matches(char::is_numeric)
        .collect::<Vec<&str>>()
        .join("")
        .parse().unwrap();
    return thread_id;
}
