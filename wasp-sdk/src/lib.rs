pub use bytes;
pub use bytes::Bytes;
pub use protobuf;
pub use protobuf::Message as TMessage;

pub use message::{Message, MessageType};
pub use wasp_macros::handler;

pub mod errors;
pub mod guest;
mod message;
