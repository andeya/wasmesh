use std::convert::TryFrom;
use std::ops::Deref;

use hyper::{Body, Client, Request, Response, Result};
use hyper::client::{Builder, HttpConnector};
use hyper::client::connect::dns::GaiResolver;
use lazy_static::lazy_static;

lazy_static! { static ref CLIENT: Client<HttpConnector<GaiResolver>, Body> = Builder::default().build_http();}


pub(crate) fn do_request(req: wasp::Request) -> Result<wasp::Response> {
    println!("got req = {:?}", req);
    let cli = CLIENT.clone();
    tokio::runtime::Builder::new_current_thread()
        .enable_io()
        .build()
        .unwrap()
        .block_on(async {
            let req = to_http_request(req);
            let resp = to_response(cli.request(req).await.unwrap()).await;
            println!("got resp = {:?}", resp);
            resp
        })
}

async fn to_response(mut resp: Response<Body>) -> Result<wasp::Response> {
    let mut r = wasp::Response::new();
    r.set_status(resp.status().as_u16() as i32);
    r.set_headers(resp.headers().iter().map(|kv| {
        (kv.0.to_string(), kv.1.to_str().unwrap().to_string())
    }).collect());
    let body = hyper::body::to_bytes(resp.body_mut()).await?;
    r.set_body(body);
    Ok(r)
}

fn to_http_request(req: wasp::Request) -> Request<Body> {
    let mut builder = Request::builder()
        .method(Method(req.method))
        .uri(req.uri);
    for x in req.headers {
        builder = builder.header(x.0.as_str(), x.1.as_str());
    }
    builder.body(Body::from(req.body)).unwrap()
}


pub(crate) fn to_http_response(mut msg: wasp::Response) -> Response<Body> {
    let mut resp = Response::builder();
    for x in msg.headers.iter() {
        resp = resp.header(x.0, x.1);
    }
    if msg.status <= 0 {
        msg.status = 200
    }
    resp = resp.status(msg.status as u16);
    resp.body(Body::from(msg.body)).unwrap()
}

pub(crate) async fn to_request(req: Request<Body>) -> wasp::Request {
    let mut msg = wasp::Request::new();
    msg.set_uri(req.uri().to_string());
    msg.set_method(Method::from(req.method()).into());
    let (parts, body) = req.into_parts();
    let body = hyper::body::to_bytes(body).await.map_or_else(|_| wasp::Bytes::new(), |v| v);
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


struct Method(wasp::Method);

impl Deref for Method {
    type Target = wasp::Method;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<wasp::Method> for Method {
    fn into(self) -> wasp::Method {
        self.0
    }
}

impl TryFrom<Method> for hyper::Method {
    type Error = hyper::http::Error;

    fn try_from(method: Method) -> std::result::Result<Self, Self::Error> {
        Ok(match method.0 {
            wasp::Method::GET => hyper::Method::GET,
            wasp::Method::HEAD => { hyper::Method::HEAD }
            wasp::Method::POST => { hyper::Method::POST }
            wasp::Method::PUT => { hyper::Method::PUT }
            wasp::Method::DELETE => { hyper::Method::DELETE }
            wasp::Method::CONNECT => { hyper::Method::CONNECT }
            wasp::Method::OPTIONS => { hyper::Method::OPTIONS }
            wasp::Method::TRACE => { hyper::Method::TRACE }
            wasp::Method::PATCH => { hyper::Method::PATCH }
            wasp::Method::ONEWAY => { hyper::Method::GET }
        })
    }
}

impl From<&hyper::Method> for Method {
    fn from(method: &hyper::Method) -> Self {
        Method(match method.clone() {
            hyper::Method::GET => { wasp::Method::GET }
            hyper::Method::HEAD => { wasp::Method::HEAD }
            hyper::Method::POST => { wasp::Method::POST }
            hyper::Method::PUT => { wasp::Method::PUT }
            hyper::Method::DELETE => { wasp::Method::DELETE }
            hyper::Method::CONNECT => { wasp::Method::CONNECT }
            hyper::Method::OPTIONS => { wasp::Method::OPTIONS }
            hyper::Method::TRACE => { wasp::Method::TRACE }
            hyper::Method::PATCH => { wasp::Method::PATCH }
            _ => { wasp::Method::GET }
        })
    }
}
