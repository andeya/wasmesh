use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::thread_local;

use wasmesh::*;

use crate::proto::resize_with_capacity;

struct Client {
    tcp_streams: HashMap<SocketAddr, TcpStream>,
}

impl Client {
    fn new() -> Client {
        Client { tcp_streams: HashMap::new() }
    }
    fn connect(&mut self, addr: SocketAddr) -> std::io::Result<&mut TcpStream> {
        let ref mut tcp_streams = self.tcp_streams;
        if !tcp_streams.contains_key(&addr) {
            tcp_streams.insert(addr.clone(), TcpStream::connect(&addr)?);
        }
        return Ok(tcp_streams.get_mut(&addr).unwrap())
    }
    fn request(&mut self, addr: SocketAddr, _seqid: i32, method: Method, msg_vec: &mut Vec<u8>) -> std::io::Result<bool> {
        let tcp_stream = self.connect(addr)?;
        Self::send(tcp_stream, msg_vec).and_then(|len_vec| {
            if let Method::ONEWAY = method {
                #[cfg(debug_assertions)] { println!("send request pack len={}, wait response", msg_vec.len()); }
                Ok(false)
            } else {
                #[cfg(debug_assertions)] { println!("send request pack len={}, wait response", msg_vec.len()); }
                Self::recv(tcp_stream, msg_vec, len_vec)?;
                Ok(true)
            }
        }).or_else(|e| {
            self.tcp_streams.remove(&addr);
            return Err(e)
        })
    }
    // send request
    fn send(tcp_stream: &mut TcpStream, msg_vec: &mut Vec<u8>) -> std::io::Result<[u8; 4]> {
        let len_vec = (msg_vec.len() as i32).to_le_bytes();
        tcp_stream.write_all(&len_vec)?;
        tcp_stream.write_all(&msg_vec)?;
        tcp_stream.flush()?;
        Ok(len_vec)
    }
    // receive response
    fn recv(tcp_stream: &mut TcpStream, msg_vec: &mut Vec<u8>, mut len_vec: [u8; 4]) -> std::io::Result<()> {
        tcp_stream.read_exact(&mut len_vec)?;
        resize_with_capacity(msg_vec, i32::from_le_bytes(len_vec) as usize);
        tcp_stream.read_exact(msg_vec)?;
        Ok(())
    }
}

thread_local!(static LOCAL_CLIENT: RefCell<Client> = RefCell::new(Client::new()));

pub(crate) fn do_request(req: Request, msg_vec: &mut Vec<u8>) -> anyhow::Result<Option<Response>> {
    // request
    #[cfg(debug_assertions)] { println!("got req = {:?}", req); }
    let addr = req.parse_uri()?.authority().unwrap().to_string().parse()?;

    LOCAL_CLIENT.with(|client| {
        let has_resp = client.borrow_mut().request(addr, req.get_seqid(), req.get_method(), msg_vec)?;
        if has_resp {
            let resp = Response::parse_from_bytes(msg_vec)?;
            #[cfg(debug_assertions)] { println!("got resp = {:?}", resp); }
            Ok(Some(resp))
        } else { Ok(None) }
    })
}
