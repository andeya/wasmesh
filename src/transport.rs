use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;

use wasp::*;

use crate::instance;
use crate::proto::{ServeOpt, write_to_vec};

trait Transport {
    fn serve(addr: SocketAddr) -> Pin<Box<dyn Future<Output=anyhow::Result<()>>>>;
    fn do_request(req: Request, msg_vec: &mut Vec<u8>) -> anyhow::Result<Option<Response>>;
}

struct HttpTransport();

impl Transport for HttpTransport {
    fn serve(addr: SocketAddr) -> Pin<Box<dyn Future<Output=anyhow::Result<()>>>> {
        Box::pin(crate::http::serve(addr))
    }

    fn do_request(req: Request, msg_vec: &mut Vec<u8>) -> anyhow::Result<Option<Response>> {
        crate::http::do_request(req, msg_vec).and_then(|r| Ok(Some(r)))
    }
}

struct RpcTransport();

impl Transport for RpcTransport {
    fn serve(addr: SocketAddr) -> Pin<Box<dyn Future<Output=anyhow::Result<()>>>> {
        Box::pin(crate::rpc::serve(addr))
    }

    fn do_request(req: Request, msg_vec: &mut Vec<u8>) -> anyhow::Result<Option<Response>> {
        crate::rpc::do_request(req, msg_vec)
    }
}

pub fn serve(serve_options: ServeOpt) -> anyhow::Result<()> {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.worker_threads(serve_options.get_worker_threads());
    builder.enable_all()
           .build()
           .unwrap()
           .block_on(async {
               instance::rebuild(&serve_options).await.unwrap();
               let (_http_res, _rpc_res) = tokio::join!(
                   async {
                       match serve_options.parse_http_addr() {
                           Ok(Some(addr))  => HttpTransport::serve(addr).await.map_err(|e|{
                               eprintln!("{}", e);
                           }).unwrap(),
                           Err(e) => eprintln!("{}", e),
                           _ => (),
                       }
                   },
                   async {
                       match serve_options.parse_rpc_addr() {
                           Ok(Some(addr))  => RpcTransport::serve(addr).await.map_err(|e|{
                               eprintln!("{}", e);
                           }).unwrap(),
                           Err(e) => eprintln!("{}", e),
                           _ => (),
                       }
                   },
               );
               Ok(())
           })
}

#[inline]
pub(crate) fn do_request_from_vec(buffer: &mut Vec<u8>) -> anyhow::Result<usize> {
    let req = wasp::Request::parse_from_bytes(buffer)?;
    do_request(req, buffer)
}

#[inline]
fn do_request(req: Request, buffer: &mut Vec<u8>) -> anyhow::Result<usize> {
    let _resp = match req.get_scheme()? {
        Scheme::HTTP | Scheme::HTTPS => {
            HttpTransport::do_request(req, buffer)
        },
        Scheme::RPC => RpcTransport::do_request(req, buffer),
        Scheme::WNS => unimplemented!(),
    };
    Ok(buffer.len())
}

#[inline]
pub fn request(req: Request) -> anyhow::Result<usize> {
    let mut buffer = Vec::new();
    let _size = write_to_vec(&req, &mut buffer);
    do_request(req, &mut buffer)
}
