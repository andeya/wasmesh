use std::convert::{TryFrom, TryInto};
use std::fmt::{Display, Formatter};
use std::panic;

pub use bytes;
pub use bytes::Bytes;
pub use guest::do_request;
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

impl TryFrom<&str> for Scheme {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_i32(
            panic::catch_unwind(|| Self::enum_descriptor_static()
                .value_by_name(value.to_uppercase().as_str())
                .value())
                .unwrap_or(-1)
        ).ok_or(anyhow::Error::msg(format!("unknown scheme: {}", value)))
    }
}

impl Request {
    pub fn get_scheme(&self) -> anyhow::Result<Scheme> {
        let s = self.uri.splitn(2, "://").next().unwrap();
        s.try_into()
    }
}

#[cfg(test)]
mod tests {
    use crate::Scheme;

    #[test]
    fn scheme() {
        assert_eq!("wns", Scheme::WNS.to_string())
    }
}
