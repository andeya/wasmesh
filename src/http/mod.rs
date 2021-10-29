use std::convert::TryFrom;
use std::ops::Deref;

pub(crate) use client::do_request;
pub(crate) use server::serve;

mod client;
mod server;

struct Method(wasp::Method);

impl Method {
    pub fn as_str(&self) -> &'static str {
        wasp::ProtobufEnum::descriptor(&self.0).name()
    }
}

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

impl From<hyper::Method> for Method {
    fn from(method: hyper::Method) -> Self {
        Method(match method {
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
