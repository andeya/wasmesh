use std::convert::TryFrom;
use std::ops::Deref;

pub(crate) use client::do_request;
pub(crate) use server::serve;

mod client;
mod server;

struct Method(wasmesh::Method);

impl Method {
    pub fn as_str(&self) -> &'static str {
        wasmesh::ProtobufEnum::descriptor(&self.0).name()
    }
}

impl Deref for Method {
    type Target = wasmesh::Method;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Into<wasmesh::Method> for Method {
    fn into(self) -> wasmesh::Method {
        self.0
    }
}

impl TryFrom<Method> for hyper::Method {
    type Error = hyper::http::Error;

    fn try_from(method: Method) -> std::result::Result<Self, Self::Error> {
        Ok(match method.0 {
            wasmesh::Method::GET => hyper::Method::GET,
            wasmesh::Method::HEAD => { hyper::Method::HEAD }
            wasmesh::Method::POST => { hyper::Method::POST }
            wasmesh::Method::PUT => { hyper::Method::PUT }
            wasmesh::Method::DELETE => { hyper::Method::DELETE }
            wasmesh::Method::CONNECT => { hyper::Method::CONNECT }
            wasmesh::Method::OPTIONS => { hyper::Method::OPTIONS }
            wasmesh::Method::TRACE => { hyper::Method::TRACE }
            wasmesh::Method::PATCH => { hyper::Method::PATCH }
            wasmesh::Method::ONEWAY => { hyper::Method::GET }
        })
    }
}

impl From<hyper::Method> for Method {
    fn from(method: hyper::Method) -> Self {
        Method(match method {
            hyper::Method::GET => { wasmesh::Method::GET }
            hyper::Method::HEAD => { wasmesh::Method::HEAD }
            hyper::Method::POST => { wasmesh::Method::POST }
            hyper::Method::PUT => { wasmesh::Method::PUT }
            hyper::Method::DELETE => { wasmesh::Method::DELETE }
            hyper::Method::CONNECT => { wasmesh::Method::CONNECT }
            hyper::Method::OPTIONS => { wasmesh::Method::OPTIONS }
            hyper::Method::TRACE => { wasmesh::Method::TRACE }
            hyper::Method::PATCH => { wasmesh::Method::PATCH }
            _ => { wasmesh::Method::GET }
        })
    }
}
