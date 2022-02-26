use std::ops::Deref;

use bytes::Bytes;
use protobuf::ProtobufEnum;

use crate::proto::{HttpMethod, HttpRequest, HttpResponse};

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        ProtobufEnum::descriptor(self).name()
    }
}

impl Deref for HttpMethod {
    type Target = hyper::Method;

    fn deref(&self) -> &Self::Target {
        match self {
            HttpMethod::GET => { &hyper::Method::GET }
            HttpMethod::HEAD => { &hyper::Method::HEAD }
            HttpMethod::POST => { &hyper::Method::POST }
            HttpMethod::PUT => { &hyper::Method::PUT }
            HttpMethod::DELETE => { &hyper::Method::DELETE }
            HttpMethod::CONNECT => { &hyper::Method::CONNECT }
            HttpMethod::OPTIONS => { &hyper::Method::OPTIONS }
            HttpMethod::TRACE => { &hyper::Method::TRACE }
            HttpMethod::PATCH => { &hyper::Method::PATCH }
        }
    }
}

impl From<hyper::Method> for HttpMethod {
    fn from(method: hyper::Method) -> Self {
        match method {
            hyper::Method::GET => { HttpMethod::GET }
            hyper::Method::HEAD => { HttpMethod::HEAD }
            hyper::Method::POST => { HttpMethod::POST }
            hyper::Method::PUT => { HttpMethod::PUT }
            hyper::Method::DELETE => { HttpMethod::DELETE }
            hyper::Method::CONNECT => { HttpMethod::CONNECT }
            hyper::Method::OPTIONS => { HttpMethod::OPTIONS }
            hyper::Method::TRACE => { HttpMethod::TRACE }
            hyper::Method::PATCH => { HttpMethod::PATCH }
            _ => { HttpMethod::GET }
        }
    }
}

impl HttpRequest {
    pub async fn from(req: hyper::Request<hyper::Body>) -> Self {
        let mut msg = HttpRequest::new();
        msg.set_url(req.uri().to_string());
        msg.set_method(req.method().clone().into());
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
}

impl From<HttpResponse> for hyper::Response<hyper::Body> {
    fn from(mut msg: HttpResponse) -> Self {
        let mut resp = hyper::Response::builder();
        for x in msg.headers.iter() {
            resp = resp.header(x.0, x.1);
        }
        if msg.status <= 0 {
            msg.set_status(200)
        }
        resp = resp.status(msg.status as u16);
        resp.body(hyper::Body::from(msg.body)).unwrap()
    }
}
