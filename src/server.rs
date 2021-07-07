use std::ffi::OsString;
use std::net::{AddrParseError, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::sync::atomic::{AtomicI32, Ordering};

use hyper::{Body, Error, Request, Response};
use hyper::body::Bytes;
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use structopt::StructOpt;

use wasp_sdk::message::{FlatBufferBuilder, HeaderBuilder, Message, MessageBuilder, MessageType, RefMessage};

use crate::instance::{self, instance_ref, INSTANCES_COUNT};

#[derive(StructOpt, Debug, Clone)]
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

pub(crate) async fn serve(serve_options: ServeOpt) -> Result<(), anyhow::Error> {
    Server::serve(serve_options)
        .await
        .map_err(|e| anyhow::Error::msg(format!("{}", e)))
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
        let thread_id = current_thread_id() % INSTANCES_COUNT;
        let ctx_id = new_ctx_id();

        // println!("========= thread_id={}, ctx_id={}", thread_id, ctx_id);

        let (data, start) = req_to_call_msg(req).await;
        let data = &data[start..];
        let ins = instance_ref(thread_id);
        ins.call_guest_handler(thread_id as i32, ctx_id, ins.set_guest_request(ctx_id, data.to_vec()));

        let reply_msg = RefMessage::from_vec(ins.get_guest_response(ctx_id));

        // println!("========= reply_msg={:?}", reply_msg);
        Ok(msg_to_resp(reply_msg.as_ref()))
    }
}

static mut CTX_ID_COUNT: AtomicI32 = AtomicI32::new(0);

fn new_ctx_id() -> i32 {
    unsafe {
        CTX_ID_COUNT.fetch_add(1, Ordering::Relaxed)
    }
}

fn current_thread_id() -> usize {
    let thread_id: usize = format!("{:?}", ::std::thread::current().id())
        .matches(char::is_numeric)
        .collect::<Vec<&str>>()
        .join("")
        .parse().unwrap();
    return thread_id;
}

fn msg_to_resp(msg: Message) -> Response<Body> {
    let mut resp = Response::builder();
    resp = resp.status(200);
    if let Some(headers) = msg.headers() {
        for x in headers {
            let key = x.key().unwrap();
            let value = x.value().unwrap();
            if key == "status" {
                resp = resp.status(value.parse::<u16>().unwrap_or(200));
            } else {
                resp = resp.header(x.key().unwrap(), x.value().unwrap());
            }
        }
    }
    let body = msg.body().unwrap_or(&[]).to_vec();
    resp.body(Body::from(body)).unwrap()
}

async fn req_to_call_msg(req: Request<Body>) -> (Vec<u8>, usize) {
    let mut builder = FlatBufferBuilder::new();
    let mut call_builder = MessageBuilder::new(&mut builder);
    call_builder.add_seqid(rand::random());
    call_builder.add_mtype(MessageType::Call);
    call_builder.add_uri(FlatBufferBuilder::new().create_string(req.uri().to_string().as_str()));
    let (parts, body) = req.into_parts();
    let body = hyper::body::to_bytes(body).await.unwrap_or(Bytes::new());
    call_builder.add_body(FlatBufferBuilder::new().create_vector_direct(body.as_ref()));

    let mut hbuilder_vec = vec![];
    for x in parts.headers.iter() {
        let mut hbuilder = FlatBufferBuilder::new();
        let mut hbuilder = HeaderBuilder::new(&mut hbuilder);
        hbuilder.add_key(FlatBufferBuilder::new().create_string(x.0.as_str()));
        hbuilder.add_value(FlatBufferBuilder::new().create_string(x.1.to_str().unwrap_or("")));
        hbuilder_vec.push(hbuilder.finish());
    }

    let mut hsbuilder = FlatBufferBuilder::new();
    let hsoffset = hsbuilder.create_vector(&hbuilder_vec);
    call_builder.add_headers(hsoffset);

    let offset = call_builder.finish();
    builder.finish_minimal(offset);
    builder.collapse()
    // let reply_msg_bytes = builder.finished_data();
}
