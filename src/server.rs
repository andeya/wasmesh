use std::ffi::OsString;
use std::net::{AddrParseError, SocketAddr, SocketAddrV4, SocketAddrV6};

use hyper::{Body, Error, Request as HttpRequest, Response as HttpResponse};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use structopt::StructOpt;
use wasp::*;

use crate::instance::{self, local_instance_ref};

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
            return self.threads
        }
        let threads = num_cpus::get();
        if threads > 0 {
            return threads
        }
        return 1
    }
}

static mut SERVER: Server = Server::new();

pub(crate) fn serve(serve_options: ServeOpt) -> Result<(), anyhow::Error> {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.worker_threads(serve_options.get_worker_threads());
    builder.enable_all()
           .build()
           .unwrap()
           .block_on(async {
               Server::serve(serve_options)
                   .await
                   .map_err(|e| anyhow::Error::msg(format!("{}", e)))
           })
}

pub(crate) struct Server {}

impl Server {
    pub(crate) const fn new() -> Self {
        Server {}
    }

    async fn serve(serve_options: ServeOpt) -> Result<(), Box<dyn std::error::Error>> {
        let addr = serve_options.parse_addr()?;
        instance::rebuild(&serve_options).await?;
        // The closure inside `make_service_fn` is run for each connection,
        // creating a 'service' to handle requests for that specific connection.
        let make_service = make_service_fn(|socket: &AddrStream| {
            let _remote_addr = socket.remote_addr();
            async {
                // This is the `Service` that will handle the connection.
                // `service_fn` is a helper to convert a function that
                // returns a Response into a `Service`.
                Ok::<_, Error>(service_fn(|req| async {
                    let r = unsafe { &SERVER }.handle(req).await;
                    if let Err(ref e) = r {
                        eprintln!("{}", e)
                    }
                    r
                }))
            }
        });
        let srv = hyper::Server::bind(&addr).serve(make_service);
        println!("Listening on http://{}", addr);
        if let Err(e) = srv.await {
            eprintln!("SERVER error: {}", e);
        }
        Ok(())
    }

    async fn handle(&self, req: HttpRequest<Body>) -> Result<HttpResponse<Body>, String> {
        // return Ok(Response::default());
        let req = to_request(req).await;

        let (thread_id, ins) = local_instance_ref();
        let ctx_id = ins.gen_ctx_id(thread_id);
        #[cfg(debug_assertions)]
        println!("thread_id:{}, ctx_id:{}", thread_id, ctx_id);
        let buffer_len = ins.use_mut_buffer(ctx_id, req.compute_size() as usize, |buffer| {
            let size = req.compute_size() as usize;
            if size > buffer.capacity() {
                buffer.resize(size, 0);
            }
            unsafe { buffer.set_len(size) };
            let mut os = CodedOutputStream::bytes(buffer);
            req.write_to_with_cached_sizes(&mut os)
               .or_else(|e| Err(format!("{}", e))).unwrap();
            buffer.len()
        });

        ins.call_guest_handler(ctx_id, buffer_len as i32);
        // println!("========= thread_id={}, ctx_id={}", thread_id, ctx_id);

        let buffer = ins.take_buffer(ctx_id).unwrap_or(vec![]);
        let resp = Response::parse_from_bytes(buffer.as_slice()).unwrap();

        ins.try_reuse_buffer(thread_id, buffer);
        // println!("========= resp={:?}", resp);
        Ok(to_http_response(resp))
    }
}

fn to_http_response(mut msg: Response) -> HttpResponse<Body> {
    let mut resp = HttpResponse::builder();
    for x in msg.headers.iter() {
        resp = resp.header(x.0, x.1);
    }
    if msg.status <= 0 {
        msg.status = 200
    }
    resp = resp.status(msg.status as u16);
    resp.body(Body::from(msg.body)).unwrap()
}

async fn to_request(req: HttpRequest<Body>) -> Request {
    let mut msg = Request::new();
    msg.set_uri(req.uri().to_string());
    msg.set_seqid(rand::random());
    let (parts, body) = req.into_parts();
    let body = hyper::body::to_bytes(body).await.map_or_else(|_| Bytes::new(), |v| v);

    for x in parts.headers.iter() {
        msg.headers.insert(
            x.0.to_string(),
            x.1
             .to_str()
             .map_or_else(|_| String::new(), |s| s.to_string()),
        );
    }
    msg.set_body(body);
    msg
}
