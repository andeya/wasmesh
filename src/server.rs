use std::collections::HashMap;
use std::ffi::OsString;
use std::net::{AddrParseError, SocketAddr, SocketAddrV4, SocketAddrV6};

use hyper::{Body, Error, Request, Response, Version};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use rand::Rng;
use structopt::StructOpt;

use wasp_sdk::proto::{RequestData, ResponseData};

use crate::wasi::Instance;

#[derive(StructOpt, Debug)]
pub struct ServeOpt {
    pub(crate) addr: String,
    pub(crate) command: String,
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
}

static mut SERVER: Server = Server::new();

pub(crate) fn serve(serve_options: ServeOpt) -> Result<(), anyhow::Error> {
    Server::serve(serve_options)
        .map_err(|e| anyhow::Error::msg(format!("{}", e)))
}

pub(crate) struct Server {
    instances: Vec<Instance>,
}

impl Server {
    pub(crate) const fn new() -> Self {
        Server {
            instances: vec![],
        }
    }

    fn serve(serve_options: ServeOpt) -> Result<(), Box<dyn std::error::Error>> {
        let addr = serve_options.parse_addr()?;
        let count = usize::max(num_cpus::get(), 1);
        unsafe {
            let ins = Instance::new(&serve_options)?;
            for _ in 0..count - 1 {
                SERVER.instances.push(ins.clone());
            }
            SERVER.instances.push(ins.clone());
            println!("num_cpus={}==========instances={}", count, SERVER.instances.len());
        }
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
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let srv = hyper::Server::bind(&addr).serve(make_service);
            println!("Listening on http://{}", addr);
            if let Err(e) = srv.await {
                eprintln!("SERVER error: {}", e);
            }
        });
        Ok(())
    }

    async fn handle(&'static self, req: Request<Body>) -> Result<Response<Body>, String> {
        let req = new_request_data(req).await;
        let b = serde_json::to_vec(&req).or_else(|e| Err(format!("{}", e)))?;
        let mut rng = rand::thread_rng();
        let i: usize = rng.gen_range(0..self.instances.len());
        println!("call=======instances[{}]========", i);
        self.instances.get(i).unwrap()
            .std_write(b)
            .and_then(Instance::call)
            .and_then(|instance| instance.std_read(|r| {
                let r: ResponseData = serde_json::from_reader(r)?;
                Ok(new_response(r))
            }))
            .or_else(|e| Err(format!("{}", e)))
    }
}


async fn new_request_data(req: Request<Body>) -> RequestData {
    let (parts, body) = req.into_parts();
    let body = hyper::body::to_bytes(body).await.map_or_else(|_| vec![], |v| v.to_vec());
    let mut headers = HashMap::new();
    for x in parts.headers.iter() {
        headers.insert(
            x.0.to_string(),
            x.1
             .to_str()
             .map_or_else(|_| String::new(), |s| s.to_string()),
        );
    }
    RequestData {
        body,
        method: parts.method.as_str().to_string(),
        uri: parts.uri.to_string(),
        version: format!("{:?}", parts.version),
        headers,
    }
}

fn new_response(data: ResponseData) -> Response<Body> {
    let mut resp = Response::builder()
        .status(data.status)
        .version(match data.version.as_str() {
            "HTTP/0.9" => Version::HTTP_09,
            "HTTP/1.0" => Version::HTTP_10,
            "HTTP/1.1" => Version::HTTP_11,
            "HTTP/2.0" => Version::HTTP_2,
            "HTTP/3.0" => Version::HTTP_3,
            _ => Version::HTTP_11,
        });
    for x in data.headers.iter() {
        resp = resp.header(x.0, x.1);
    }
    resp.body(Body::from(data.body)).unwrap()
}
