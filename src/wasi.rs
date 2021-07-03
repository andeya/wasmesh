use std::io::Write;

use wasmer::{Function, FunctionType, import_namespace, ImportObject, Memory, MemoryView, Module, NativeFunc, Store};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;
use wasmer_wasi::{Pipe, WasiEnv, WasiFile, WasiState};

use crate::server::ServeOpt;

#[derive(Clone, Debug)]
pub(crate) struct Instance {
    pub instance: wasmer::Instance,
    pub wasi_env: WasiEnv,
}

impl Instance {
    pub(crate) fn new(serve_options: &ServeOpt) -> Result<Instance, Box<dyn std::error::Error>> {
        // Create a Store.
        // Note that we don't need to specify the engine/compiler if we want to use
        // the default provided by Wasmer.
        // You can use `Store::default()` for that.
        let store = Store::new(&Universal::new(Cranelift::default()).engine());

        println!("Compiling module...");

        // Let's compile the Wasm module.
        let module = Module::from_file(&store, serve_options.get_wasm_path())?;

        println!("Module exports functions: {:?}", module.exports().functions().collect::<Vec<wasmer::ExportType<FunctionType>>>());

        // First, we create the `WasiEnv` with the stdio pipes
        let input = Pipe::new();
        let output = Pipe::new();
        let mut wasi_env = WasiState::new(serve_options.get_name())
            .preopen_dirs(serve_options.get_preopen_dirs())?
            .args(serve_options.to_args_unchecked())
            .stdin(Box::new(input))
            .stdout(Box::new(output))
            .finalize()?;

        println!("Instantiating module with WASI imports...");

        // Then, we get the import object related to our WASI
        // and attach it to the Wasm instance.
        let mut import_object = wasi_env.import_object(&module)?;
        Instance::register_import_object(&mut import_object, &store);
        let instance = wasmer::Instance::new(&module, &import_object)?;

        println!("Created instance: {}", serve_options.command);

        Ok(Instance {
            instance,
            wasi_env,
        }.init())
    }
    // TODO
    fn register_import_object(import_object: &mut ImportObject, store: &Store) {
        import_object.register("env", import_namespace!({
            "_wasp_send_msg" => Function::new_native(store, |offset: i32| -> i32{0}),
            "_wasp_recall_msg_size" => Function::new_native(store, || -> i32{0}),
            "_wasp_recall_msg_data" => Function::new_native(store, |offset: i32|{}),
        }));
    }
    fn init(self) -> Self {
        let view = self.get_view();
        Self::set_data_size(&view, 0);
        self
    }
    fn get_wasp_handler(&self) -> NativeFunc<i32> {
        self.instance.exports.get_native_function::<(i32), ()>("_wasp_handler").unwrap()
    }
    fn get_memory(&self) -> &Memory {
        self.instance.exports.get_memory("memory").unwrap()
    }
    fn get_view(&self) -> MemoryView<u8> {
        self.get_memory().view::<u8>()
    }
    fn set_view_bytes<'a>(view: &MemoryView<u8>, offset: usize, data: impl IntoIterator<Item=&'a u8> + ExactSizeIterator) {
        for (cell, b) in view[offset..offset + data.len()].iter().zip(data) {
            cell.set(*b);
        }
    }
    fn get_view_bytes(view: &MemoryView<u8>, offset: usize, size: usize) -> Vec<u8> {
        view[offset..offset + size]
            .iter()
            .map(|c| c.get())
            .collect()
    }
    fn set_data_size(view: &MemoryView<u8>, size: u32) {
        // Fill the first 4 bytes with 0 as a place to record the message length
        Self::set_view_bytes(view, 1, u32::to_be_bytes(size).iter())
    }
    fn get_data_size(view: &MemoryView<u8>) -> usize {
        // Setup the 4 bytes that will be converted
        // into our new length
        let mut new_len_bytes = [0u8; 4];
        for i in 0..4 {
            new_len_bytes[i] = view.get(i + 1).map(|c| c.get()).unwrap_or(0);
        }
        u32::from_ne_bytes(new_len_bytes) as usize
    }
    fn set_data(&self, offset: usize, data: Vec<u8>) {
        let view = self.get_view();
        Self::set_data_size(&view, data.len() as u32);
        Self::set_view_bytes(&view, offset, data.iter())
    }
    fn get_data(&self, offset: usize) -> Vec<u8> {
        let view = self.get_view();
        Self::get_view_bytes(&view, offset, Self::get_data_size(&view))
    }

    pub(crate) fn call(&self) -> Result<&Self, Box<dyn std::error::Error>> {
        println!("Call WASI `_wasp_serve_event` function...");
        // And we just call the `_wasp_serve_event` function!
        let start = self.instance.exports.get_function("_wasp_serve_event")?;
        start.call(&[])?;
        Ok(self)
    }
    pub(crate) fn std_write(&self, data: Vec<u8>) -> Result<&Self, Box<dyn std::error::Error>> {
        let data = data.as_slice();
        println!("Writing \"{}\" to the WASI stdin...", String::from_utf8_lossy(data));
        // To write to the stdin, we need a mutable reference to the pipe
        let mut state = self.wasi_env.state();
        let wasi_stdin = state.fs.stdin_mut()?.as_mut().unwrap();
        wasi_stdin.write_all(data)?;
        println!("Write to the WASI stdin!");
        Ok(self)
    }
    pub(crate) fn std_read<F, T>(&self, read_fn: F) -> Result<T, Box<dyn std::error::Error>>
        where F: FnOnce(&mut Box<dyn WasiFile>) -> Result<T, Box<dyn std::error::Error>>
    {
        println!("Reading from the WASI stdout...");
        // To read from the stdout, we again need a mutable reference to the pipe
        let mut state = self.wasi_env.state();
        let wasi_stdout = state.fs.stdout_mut()?.as_mut().unwrap();
        read_fn(wasi_stdout)
    }
}
