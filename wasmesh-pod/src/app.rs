use std::cell::RefCell;
use std::ffi::OsString;
use std::net::{AddrParseError, SocketAddr, SocketAddrV4, SocketAddrV6};

use structopt::StructOpt;
use wasmy_vm::{load_wasm, WasmInfo};

use crate::http;
// make sure submit runtime handlers
#[allow(unused_imports)]use crate::runtime as _;

#[derive(StructOpt, Debug, Clone)]
pub struct ServeOpt {
    /// wasm server file path
    pub(crate) wasm: String,
    /// HTTP listening address
    // #[structopt(long, default_value = "0.0.0.0:9090")]
    #[structopt(long)]
    pub(crate) http: Option<String>,
    /// RPC listening address
    // #[structopt(long, default_value = "0.0.0.0:9091")]
    // #[structopt(long)]
    // pub(crate) rpc: Option<String>,
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
    // pub(crate) fn parse_rpc_addr(&self) -> Result<Option<SocketAddr>, AddrParseError> {
    //     Self::parse_addr(self.rpc.as_ref())
    // }
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
        &self.wasm
    }
    pub(crate) fn get_wasm_path(&self) -> &String {
        &self.wasm
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

thread_local! {
    static WASM_INFO: RefCell<WasmInfo> = RefCell::new(WasmInfo{wasm_path:"".to_string()});
}

pub fn serve(serve_options: ServeOpt) -> anyhow::Result<()> {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.worker_threads(serve_options.get_worker_threads());
    builder.enable_all()
           .build()?
        .block_on(async {
            WASM_INFO.with(|wi| {
                let info = WasmInfo { wasm_path: serve_options.get_wasm_path().clone() };
                wi.replace(info.clone());
                load_wasm(info).unwrap_or_else(|e| eprintln!("{}", e));
            });
            tokio::join!(
                   async {
                       match serve_options.parse_http_addr() {
                           Ok(Some(addr))  => http::serve(&WASM_INFO, addr).await.map_err(|e|{
                               eprintln!("{}", e);
                           }).unwrap(),
                           Err(e) => eprintln!("{}", e),
                           _ => (),
                       }
                   },
                   // async {
                   //     match serve_options.parse_rpc_addr() {
                   //         Ok(Some(addr))  => RpcTransport::serve(addr).await.map_err(|e|{
                   //             eprintln!("{}", e);
                   //         }).unwrap(),
                   //         Err(e) => eprintln!("{}", e),
                   //         _ => (),
                   //     }
                   // },
               );
            Ok(())
        })
}
