use std::io::{Read, Write};
use std::net::TcpStream;

use wasp::*;

use crate::proto::resize_with_capacity;

pub(crate) fn do_request(req: Request, msg_vec: &mut Vec<u8>) -> anyhow::Result<Response> {
    // request
    #[cfg(debug_assertions)] { println!("got req = {:?}", req); }
    let addr = req.parse_uri()?.authority().unwrap().to_string();
    let mut stream = TcpStream::connect(addr)?;
    let mut len_vec = (msg_vec.len() as i32).to_le_bytes();
    stream.write_all(&len_vec)?;
    stream.write_all(&msg_vec)?;
    stream.flush()?;

    #[cfg(debug_assertions)] { println!("send request pack len={}, wait response", msg_vec.len()); }

    // response
    stream.read_exact(&mut len_vec)?;
    resize_with_capacity(msg_vec, i32::from_le_bytes(len_vec) as usize);
    stream.read_exact(msg_vec)?;
    let resp = Response::parse_from_bytes(msg_vec)?;
    #[cfg(debug_assertions)] { println!("got resp = {:?}", resp); }
    Ok(resp)
}
