extern crate bytes;
extern crate protobuf;

pub use bytes::Bytes;
use protobuf::ProtobufEnum;

pub use proto::*;
pub use wasmy_abi::*;

mod proto;
mod http_method;
mod http_request;

impl From<WasmMethod> for wasmy_abi::Method {
    fn from(m: WasmMethod) -> Self {
        m.value()
    }
}

impl From<VmMethod> for wasmy_abi::Method {
    fn from(m: VmMethod) -> Self {
        m.value()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
