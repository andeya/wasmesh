use std::collections::HashMap;

use serde::{Deserialize, Serialize};

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

