use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};

pub use anyhow;
pub use bytes;
pub use bytes::Bytes;
pub use guest::do_request;
pub use http::uri;
pub use http::uri::Uri;
pub use message::{Method, Request, Response, Scheme};
pub use protobuf;
pub use protobuf::{CodedOutputStream, Message};
use protobuf::ProtobufEnum;
pub use wasp_macros::handler;

pub mod errors;
pub mod guest;
mod message;

impl Display for Scheme {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.descriptor().name().to_lowercase().as_str())
    }
}

impl From<&uri::Scheme> for Scheme {
    fn from(scheme: &uri::Scheme) -> Self {
        scheme.as_str().try_into().unwrap()
    }
}

impl TryFrom<&str> for Scheme {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value.to_uppercase().as_str() {
            "HTTP" | "/" => Ok(Scheme::HTTP),
            "HTTPS" => Ok(Scheme::HTTPS),
            "RPC" => Ok(Scheme::RPC),
            "Scheme::WNS" => Ok(Scheme::WNS),
            _ => Err(anyhow::Error::msg(format!("unknown scheme: {}", value))),
        }
    }
}

impl Request {
    pub fn get_scheme(&self) -> anyhow::Result<Scheme> {
        let s = self.uri.splitn(2, "://").next().unwrap();
        s.try_into()
    }
    pub fn parse_uri(&self) -> anyhow::Result<Uri> {
        self.uri.as_str().parse::<Uri>().map_err(|e| anyhow::Error::new(e))
    }
}

#[cfg(test)]
mod tests {
    use url::Url;

    use crate::{Request, Scheme};

    #[test]
    fn scheme() {
        assert_eq!("wns", Scheme::WNS.to_string());
        let mut req = Request::new();
        req.set_uri("/".to_string());
        println!("{:?}", req.get_scheme());
    }

    #[test]
    fn parse_uri() {
        let uri = Url::parse("rpc://127.0.0.1:8080/a/b?c=0#d").unwrap();
        assert_eq!("127.0.0.1", uri.host().unwrap().to_string());
        assert_eq!(Some(8080u16), uri.port())
    }
}
