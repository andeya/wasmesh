use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

use wasmer::{Function, FunctionType, import_namespace, ImportObject, Memory, MemoryView, Module, Store};
#[cfg(feature = "cranelift")]
use wasmer_compiler_cranelift::Cranelift;
#[cfg(feature = "llvm")]
use wasmer_compiler_llvm::LLVM;
use wasmer_engine_universal::Universal;
use wasmer_wasi::{WasiEnv, WasiState};

use crate::proto::{resize_with_capacity, ServeOpt};

#[derive(Clone, Debug)]
pub(crate) struct Instance {
    instance: wasmer::Instance,
    message_cache: RefCell<HashMap<i64, Vec<u8>>>,
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
            "_wasmesh_recall_request" => Function::new_native(store, |ctx_id: i64, offset: i32| {
                recall_data_from_buffer(ctx_id, offset)
            }),
            "_wasmesh_send_response" => Function::new_native(store, |ctx_id: i64, offset: i32, size: i32| {
                let thread_id = Instance::get_thread_id_from_ctx_id(ctx_id);
                #[cfg(debug_assertions)]
                println!("_wasmesh_send_response: thread_id:{}, ctx_id:{}, offset:{}", thread_id, ctx_id, offset);
                let ins = instance_ref(thread_id as usize);
                let _ = ins.use_mut_buffer(ctx_id, size as usize, |buffer|{
                    ins.read_view_bytes(offset as usize, size as usize, buffer);
                    buffer.len()
                });
            }),
            "_wasmesh_send_request" => Function::new_native(store, |ctx_id: i64, offset: i32, size: i32|-> i32 {
                let thread_id = Instance::get_thread_id_from_ctx_id(ctx_id);
                #[cfg(debug_assertions)]
                println!("_wasmesh_send_request: thread_id:{}, ctx_id:{}, offset:{}, size:{}", thread_id, ctx_id, offset, size);
                let ins = instance_ref(thread_id as usize);
                ins.use_mut_buffer(ctx_id, size as usize, |mut buffer|{
                    ins.read_view_bytes(offset as usize, size as usize, buffer);
                    crate::transport::do_request_from_vec(&mut buffer).unwrap()
                }) as i32
            }),
            "_wasmesh_recall_response" => Function::new_native(store, |ctx_id: i64, offset: i32| {
                recall_data_from_buffer(ctx_id, offset)
            }),
        }));
    }
    fn init(self) -> Self {
        self
    }
    pub(crate) fn use_mut_buffer<F: FnOnce(&mut Vec<u8>) -> usize>(&self, ctx_id: i64, size: usize, call: F) -> usize {
        let mut cache = self.message_cache.borrow_mut();
        if let Some(buffer) = cache.get_mut(&ctx_id) {
            if size > 0 {
                resize_with_capacity(buffer, size);
            }
            return call(buffer);
        }
        cache.insert(ctx_id, vec![0; size]);
        call(cache.get_mut(&ctx_id).unwrap())
    }
    pub(crate) fn take_buffer(&self, ctx_id: i64) -> Option<Vec<u8>> {
        self.message_cache.borrow_mut().remove(&ctx_id)
    }
    pub(crate) fn call_guest_handler(&self, ctx_id: i64, size: i32) {
        loop {
            if let Err(e) = self
                .instance
                .exports
                .get_native_function::<(i64, i32), ()>("_wasmesh_guest_handler")
                .unwrap()
                .call(ctx_id, size)
            {
                let estr = format!("{:?}", e);
                eprintln!("call _wasmesh_guest_handler error: {}", estr);
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
    fn read_view_bytes(&self, offset: usize, size: usize, buffer: &mut Vec<u8>) {
        // println!("read_view_bytes: offset:{}, size:{}", offset, size);
        if size == 0 {
            resize_with_capacity(buffer, size);
            return;
        }
        let view = self.get_view();
        for x in view[offset..(offset + size)]
            .iter()
            .map(|c| c.get()).enumerate() {
            buffer[x.0] = x.1;
        }
    }
    pub(crate) fn gen_ctx_id(&self, thread_id: usize) -> i64 {
        (thread_id as i64) << 32 |
            (self.ctx_id_count.replace_with(|v| *v + 1) as i64)
    }
    fn next_ctx_id(&self, thread_id: usize) -> i64 {
        (thread_id as i64) << 32 |
            ((self.ctx_id_count.borrow_mut().clone() + 1) as i64)
    }
    #[inline]
    fn get_thread_id_from_ctx_id(ctx_id: i64) -> usize {
        (ctx_id >> 32) as usize
    }
    // fn split_ctx_id(ctx_id: i64) -> (usize, i32) {
    //     ((ctx_id >> 32) as usize, (ctx_id << 32 >> 32) as i32)
    // }
    pub(crate) fn try_reuse_buffer(&self, thread_id: usize, buffer: Vec<u8>) {
        let next_id = self.next_ctx_id(thread_id);
        let mut cache = self.message_cache.borrow_mut();
        if !cache.contains_key(&next_id) {
            cache.insert(next_id, buffer);
        }
    }
}

#[test]
fn test_ctx_id() {
    let thread_id = 12;
    let buf_idx = 110;
    let ctx_id = (thread_id as i64) << 32 | (buf_idx as i64);
    println!("{}", ctx_id);
    let x = ((ctx_id >> 32) as usize, (ctx_id << 32 >> 32) as i32);
    assert_eq!(x.0, thread_id);
    assert_eq!(x.1, buf_idx);
}

fn current_thread_id() -> usize {
    let thread_id: usize = format!("{:?}", ::std::thread::current().id())
        .matches(char::is_numeric)
        .collect::<Vec<&str>>()
        .join("")
        .parse().unwrap();
    return thread_id;
}

fn recall_data_from_buffer(ctx_id: i64, offset: i32) {
    let thread_id = Instance::get_thread_id_from_ctx_id(ctx_id);
    #[cfg(debug_assertions)]
    println!("_wasmesh_recall_response: thread_id:{}, ctx_id:{}, offset:{}", thread_id, ctx_id, offset);
    let ins = instance_ref(thread_id);
    let _ = ins.use_mut_buffer(ctx_id, 0, |data| {
        ins.set_view_bytes(offset as usize, data.iter());
        let len = data.len();
        unsafe { data.set_len(0) };
        len
    });
}
