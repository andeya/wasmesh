use std::collections::HashMap;
use std::convert::{From, TryFrom};
use std::fmt;
use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};
use serde_repr::*;

use crate::errors::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestData {
    pub method: String,

    /// The request's URI
    pub uri: String,

    /// The request's version
    pub version: String,

    /// The request's headers
    pub headers: HashMap<String, String>,

    pub body: Vec<u8>,
}

impl RequestData {
    pub fn from_reader(r: impl std::io::Read) -> serde_json::Result<Self> {
        serde_json::from_reader(r)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseData {
    pub status: u16,

    /// The request's version
    pub version: String,

    /// The request's headers
    pub headers: HashMap<String, String>,

    pub body: Vec<u8>,
}

impl ResponseData {
    pub fn from_request_data(req: RequestData, body: Vec<u8>) -> ResponseData {
        ResponseData {
            status: 200,
            version: req.version,
            headers: HashMap::new(),
            body: body,
        }
    }
    pub fn to_writer(&self, writer: impl std::io::Write) -> serde_json::Result<()> {
        serde_json::to_writer(writer, self)
    }
}

/// Wasp message identifier.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Message {
    /// Service call the message is associated with.
    uri: String,
    /// Message type.
    message_type: MessageType,
    /// Ordered sequence number identifying the message.
    message_seqid: i32,
    /// Content type.
    content_type: String,
    /// Message content.
    content: Vec<u8>,
}

impl Message {
    /// Create a `Message` for a Thrift service-call named `name`
    /// with message type `message_type` and sequence number `sequence_number`.
    pub fn new<S: Into<String>>(
        uri: S,
        message_type: MessageType,
        message_seqid: i32,
    ) -> Self {
        Message {
            uri: uri.into(),
            message_type,
            message_seqid,
            content_type: String::new(),
            content: vec![],
        }
    }
}

/// Wasp message types.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u8)]
pub enum MessageType {
    /// Service-call request.
    Call = 0x01u8,
    /// Service-call response.
    Reply = 0x02u8,
    /// Unexpected error in the remote service.
    Exception = 0x03u8,
    /// One-way service-call request (no response is expected).
    OneWay = 0x04u8,
}

impl Display for MessageType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
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
