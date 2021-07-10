pub use capnp::message::{Builder, HeapAllocator, ReaderOptions, ReaderSegments};
pub use capnp::serialize_packed::*;

pub use message_capnp::{header, request, response};
pub use wasp_macros::handler;

pub mod errors;
pub mod guest;
#[allow(dead_code)]
mod message_capnp;

pub fn build_request<F: FnOnce(request::Builder)>(req_setter: F) -> Builder<HeapAllocator> {
    let mut builder = Builder::new_default();
    req_setter(builder.init_root());
    builder
}

pub fn build_response<F: FnOnce(response::Builder)>(resp_setter: F) -> Builder<HeapAllocator> {
    let mut builder = Builder::new_default();
    resp_setter(builder.init_root());
    builder
}
