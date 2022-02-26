use std::ffi::OsString;
use std::net::{AddrParseError, SocketAddr, SocketAddrV4, SocketAddrV6};

use structopt::StructOpt;
use wasmesh::*;

#[derive(StructOpt, Debug, Clone)]
pub struct ServeOpt {
    /// wasm server file path
    pub(crate) command: String,
    /// HTTP listening address
    // #[structopt(long, default_value = "0.0.0.0:9090")]
    #[structopt(long)]
    pub(crate) http: Option<String>,
    /// RPC listening address
    // #[structopt(long, default_value = "0.0.0.0:9091")]
    #[structopt(long)]
    pub(crate) rpc: Option<String>,
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

#[allow(dead_code)]
impl ServeOpt {
    pub(crate) fn parse_http_addr(&self) -> Result<Option<SocketAddr>, AddrParseError> {
        Self::parse_addr(self.http.as_ref())
    }
    pub(crate) fn parse_rpc_addr(&self) -> Result<Option<SocketAddr>, AddrParseError> {
        Self::parse_addr(self.rpc.as_ref())
    }
    fn parse_addr(addr: Option<&String>) -> Result<Option<SocketAddr>, AddrParseError> {
        if addr.is_none() {
            return Ok(None)
        }
        let addr_str: &String = addr.unwrap();
        Ok(Some(addr_str.parse::<SocketAddrV4>()
                        .and_then(|a| Ok(SocketAddr::V4(a))).or_else(|_| {
            addr_str.parse::<SocketAddrV6>()
                    .and_then(|a| Ok(SocketAddr::V6(a)))
        })?))
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

pub(crate) fn write_to_vec<M: Message>(msg: &M, buffer: &mut Vec<u8>) -> usize {
    let size = msg.compute_size() as usize;
    resize_with_capacity(buffer, size);
    write_to_with_cached_sizes(msg, buffer)
}

pub(crate) fn write_to_with_cached_sizes<M: Message>(msg: &M, buffer: &mut Vec<u8>) -> usize {
    let mut os = CodedOutputStream::bytes(buffer);
    msg.write_to_with_cached_sizes(&mut os)
       .or_else(|e| Err(format!("{}", e))).unwrap();
    // os.flush().unwrap();
    buffer.len()
}

pub(crate) fn resize_with_capacity(buffer: &mut Vec<u8>, new_size: usize) {
    if new_size > buffer.capacity() {
        buffer.resize(new_size, 0);
    } else {
        unsafe { buffer.set_len(new_size) };
    }
}
