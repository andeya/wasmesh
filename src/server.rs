use std::ffi::OsString;
use std::net::{AddrParseError, SocketAddr, SocketAddrV4, SocketAddrV6};

use bytes::Bytes;
use hyper::{Body, Error, Request, Response};
use hyper::http::request::Parts;
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

    async fn handle(&self, req: Request<Body>) -> Result<Response<Body>, String> {
        // return Ok(Response::default());
        let (parts, body) = req.into_parts();
        let body = hyper::body::to_bytes(body).await.map_or_else(|_| Bytes::new(), |v| v);
        let builder = build_request(move |build| { to_request(parts, body, build) });
        let (thread_id, ins) = local_instance_ref();
        let ctx_id = ins.gen_ctx_id();
        let mut buffer = Vec::with_capacity(builder.len());
        write_message(&mut buffer, &builder).unwrap();
        ins.call_guest_handler(thread_id as i32, ctx_id, ins.set_guest_request(ctx_id, buffer));

        let buffer = ins.get_guest_response(ctx_id);
        let reader = read_message(&mut buffer.as_slice(), ReaderOptions::new()).unwrap();
        let resp_reader = reader.get_root::<response::Reader>().unwrap();
        // println!("========= resp={:?}", resp);
        let mut resp = Response::builder();
        for header in resp_reader.get_headers().unwrap() {
            resp = resp.header(header.get_key().unwrap_or(""), header.get_value().unwrap_or(""));
        }
        let mut status_code = resp_reader.get_status();
        if status_code <= 0 {
            status_code = 200
        }
        resp = resp.status(status_code as u16);
        Ok(resp.body(Body::from(resp_reader.get_body().unwrap_or(&[]).to_owned())).unwrap())
    }
}

fn to_request(parts: Parts, body: Bytes, mut build: request::Builder) {
    build.set_uri(parts.uri.to_string().as_str());
    build.set_seqid(rand::random());
    let mut headers = build.reborrow().init_headers(parts.headers.len() as u32);
    for (i, x) in parts.headers.iter().enumerate() {
        let mut header = headers.reborrow().get(i as u32);
        header.set_key(x.0.to_string().as_str());
        header.set_value(x.1.to_str().unwrap_or(""));
    }
    build.set_body(&*body);
}
