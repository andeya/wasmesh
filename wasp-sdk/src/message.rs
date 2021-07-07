pub use flatbuffers::*;

pub use crate::message_generated::message::{Header, HeaderBuilder, Message, MessageArgs, MessageBuilder, MessageType};
use crate::message_generated::message::root_as_message_unchecked;

pub struct RefMessage {
    buffer: Vec<u8>,
}

impl RefMessage {
    pub fn from_vec(buffer: Vec<u8>) -> Self {
        RefMessage { buffer }
    }
    pub fn as_ref(&self) -> Message {
        unsafe { root_as_message_unchecked(self.buffer.as_slice()) }
    }
}
