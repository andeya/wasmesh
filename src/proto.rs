use std::ffi::OsString;
use std::net::{AddrParseError, SocketAddr, SocketAddrV4, SocketAddrV6};

use structopt::StructOpt;
use wasp::*;

#[derive(StructOpt, Debug, Clone)]
pub struct ServeOpt {
    pub(crate) addr: String,
    pub(crate) command: String,
    /// worker threads, default to lazy auto-detection (one thread per CPU core)
    #[structopt(long, default_value = "0")]
    pub(crate) threads: usize,
    /// WASI pre-opened directory
    #[structopt(long = "dir", multiple = true, group = "wasi")]
    pub(crate) pre_opened_directories: Vec<String>,
    /// Application arguments
    #[structopt(multiple = true, parse(from_os_str))]
    pub(crate) args: Vec<OsString>,
}

impl ServeOpt {
    pub(crate) fn parse_addr(&self) -> Result<SocketAddr, AddrParseError> {
        let mut addr = self.addr.parse::<SocketAddrV4>()
                           .and_then(|a| Ok(SocketAddr::V4(a)));
        if addr.is_err() {
            addr = self.addr.parse::<SocketAddrV6>()
                       .and_then(|a| Ok(SocketAddr::V6(a)));
        }
        addr
    }
    pub(crate) fn get_name(&self) -> &String {
        &self.command
    }
    pub(crate) fn get_wasm_path(&self) -> &String {
        &self.command
    }
    pub(crate) fn get_preopen_dirs(&self) -> &Vec<String> {
        &self.pre_opened_directories
    }
    pub(crate) fn to_args_unchecked(&self) -> impl IntoIterator<Item=&str> {
        self.args.iter().map(|v| v.to_str().unwrap()).collect::<Vec<&str>>()
    }
    pub(crate) fn get_worker_threads(&self) -> usize {
        if self.threads > 0 {
            return self.threads;
        }
        let threads = num_cpus::get();
        if threads > 0 {
            return threads;
        }
        return 1;
    }
}

pub(crate) fn write_to_vec<M: Message>(msg: M, buffer: &mut Vec<u8>) -> usize {
    let size = msg.compute_size() as usize;
    if size > buffer.capacity() {
        buffer.resize(size, 0);
    }
    unsafe { buffer.set_len(size) };
    let mut os = CodedOutputStream::bytes(buffer);
    msg.write_to_with_cached_sizes(&mut os)
       .or_else(|e| Err(format!("{}", e))).unwrap();
    buffer.len()
}
