/// ab（apache benchmark）
/// `ab -c 100 -n 10000 http://127.0.0.1:9090/`

use std::env;

use rand::Rng;
use wasp::*;

#[wasp::handler]
fn handler(req: Request) -> Option<Response> {
    // eprintln!("Args: {:?}", env::args().collect::<Vec<String>>());
    // eprintln!("[WASI-Simple] Request: {:?}", req);
    if req.oneway {
        return None;
    }
    let mut resp = Response::new();
    resp.set_seqid(req.seqid);
    let y: u8 = 10;
    // let y: u8 = rand::thread_rng().gen();
    let body = format!("this is Response {}", "=".repeat(y as usize));
    resp.set_body(Bytes::from(body));
    // eprintln!("[WASI-Simple] Response: {:?}", resp);
    Some(resp)
}
