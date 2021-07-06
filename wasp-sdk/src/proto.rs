use std::collections::HashMap;
use std::convert::{From, TryFrom};
use std::fmt;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use serde_repr::*;

use crate::errors::*;

/// Wasp message identifier.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
pub struct Message {
    /// Service call the message is associated with.
    pub uri: String,
    /// Message type.
    pub mtype: MessageType,
    /// Ordered sequence number identifying the message.
    pub seqid: i32,
    /// Message headers.
    pub headers: HashMap<String, String>,
    /// Message body bytes.
    pub body: Vec<u8>,
}

impl Message {
    /// Create a `Message` for a Thrift service-call named `name`
    /// with message type `message_type` and sequence number `sequence_number`.
    pub fn new<S: Into<String>>(
        uri: S,
        mtype: MessageType,
        seqid: i32,
    ) -> Self {
        Message {
            uri: uri.into(),
            mtype,
            seqid,
            headers: HashMap::new(),
            body: vec![],
        }
    }
    pub fn set_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = headers;
        self
    }
    pub fn set_header<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
    pub fn set_body<T: Into<Vec<u8>>>(mut self, body: T) -> Self {
        self.body = body.into();
        self
    }
}

/// Wasp message types.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum MessageType {
    Unknown = 0x00u8,
    /// Service-call request.
    Call = 0x01u8,
    /// Service-call response.
    Reply = 0x02u8,
    /// Unexpected error in the remote service.
    Exception = 0x03u8,
    /// One-way service-call request (no response is expected).
    OneWay = 0x04u8,
}

impl Default for MessageType {
    fn default() -> Self {
        MessageType::Unknown
    }
}

impl Display for MessageType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            MessageType::Unknown => write!(f, "Unknown"),
            MessageType::Call => write!(f, "Call"),
            MessageType::Reply => write!(f, "Reply"),
            MessageType::Exception => write!(f, "Exception"),
            MessageType::OneWay => write!(f, "OneWay"),
        }
    }
}

impl From<MessageType> for u8 {
    fn from(message_type: MessageType) -> Self {
        message_type as u8
    }
}

impl TryFrom<u8> for MessageType {
    type Error = crate::errors::Error;
    fn try_from(b: u8) -> Result<Self, Self::Error> {
        match b {
            0x01 => Ok(MessageType::Call),
            0x02 => Ok(MessageType::Reply),
            0x03 => Ok(MessageType::Exception),
            0x04 => Ok(MessageType::OneWay),
            unknown => Err(crate::errors::Error::Protocol(ProtocolError {
                kind: ProtocolErrorKind::InvalidData,
                message: format!("cannot convert {} to MessageType", unknown),
            })),
        }
    }
}
