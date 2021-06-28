use std::env;
use std::io::Write;

use wasmer::{FunctionType, Module, Store};
use wasmer_compiler_cranelift::Cranelift;
use wasmer_engine_universal::Universal;
use wasmer_wasi::{Pipe, WasiEnv, WasiFile, WasiState};

pub(crate) struct Instance {
    pub instance: wasmer::Instance,
    pub wasi_env: WasiEnv,
}

impl Instance {
    pub(crate) fn new() -> Result<Instance, Box<dyn std::error::Error>> {
        let args: Vec<String> = env::args().collect();
        let wasm_path = args.get(1).ok_or("missing wasm file path argument")?;
        let wasm_bytes = std::fs::read(wasm_path)?;

        // Create a Store.
        // Note that we don't need to specify the engine/compiler if we want to use
        // the default provided by Wasmer.
        // You can use `Store::default()` for that.
        let store = Store::new(&Universal::new(Cranelift::default()).engine());

        println!("Compiling module...");
        // Let's compile the Wasm module.
        let module = Module::new(&store, wasm_bytes)?;
        let ex: Vec<wasmer::ExportType<FunctionType>> = module.exports().functions().collect();

        println!("Creating `WasiEnv`...{:?}", ex);
        // First, we create the `WasiEnv` with the stdio pipes
        let input = Pipe::new();
        let output = Pipe::new();
        let mut wasi_env = WasiState::new("hello")
            .preopen_dir("./")?
            .stdin(Box::new(input))
            .stdout(Box::new(output))
            .finalize()?;

        println!("Instantiating module with WASI imports...");
        // Then, we get the import object related to our WASI
        // and attach it to the Wasm instance.
        let import_object = wasi_env.import_object(&module)?;
        let instance = wasmer::Instance::new(&module, &import_object)?;

        println!("start serving...");

        Ok(Instance {
            instance,
            wasi_env,
        })
    }
    pub(crate) fn call(&self) -> Result<&Self, Box<dyn std::error::Error>> {
        println!("Call WASI `_start` function...");
        // And we just call the `_start` function!
        let start = self.instance.exports.get_function("_start")?;
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
