use std::collections::HashMap;
use std::net::SocketAddr;

use hyper::{Body, Error, Request, Response, Version};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use rand::Rng;

use wasp_sdk::proto::{RequestData, ResponseData};

use crate::wasi::Instance;

pub(crate) struct Server {
    instances: Vec<Instance>,
}

impl Server {
    pub(crate) fn new() -> Result<Server, Box<dyn std::error::Error>> {
        let count = usize::max(num_cpus::get(), 1);
        let mut instances = vec![];
        for _ in 0..count {
            instances.push(Instance::new()?);
        }
        Ok(Server {
            instances,
        })
    }
    pub(crate) async fn serve(&'static self, addr: SocketAddr) {
        pretty_env_logger::init();
        // The closure inside `make_service_fn` is run for each connection,
        // creating a 'service' to handle requests for that specific connection.
        let make_service = make_service_fn(|socket: &AddrStream| {
            let _remote_addr = socket.remote_addr();
            async move {
                // This is the `Service` that will handle the connection.
                // `service_fn` is a helper to convert a function that
                // returns a Response into a `Service`.
                Ok::<_, Error>(service_fn(move |req|
                    self.handle(req)
                ))
            }
        });
        let server = hyper::Server::bind(&addr).serve(make_service);
        println!("Listening on http://{}", addr);
        if let Err(e) = server.await {
            eprintln!("server error: {}", e);
        }
    }

    async fn handle(&self, req: Request<Body>) -> Result<Response<Body>, String> {
        let req = new_request_data(req).await;
        let b = serde_json::to_vec(&req).or_else(|e| Err(format!("{}", e)))?;
        let mut rng = rand::thread_rng();
        let i: usize = rng.gen_range(0..self.instances.len());
        self.instances[i]
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
