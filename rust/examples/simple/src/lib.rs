/// ab（apache benchmark）
/// `ab -c 100 -n 10000 http://127.0.0.1:9090/`

use std::env;

use rand::Rng;
use wasp::*;

#[wasp::handler]
fn handler(req: request::Reader, resp: Option<response::Builder>) {
    // eprintln!("Args: {:?}", env::args().collect::<Vec<String>>());
    // eprintln!("[WASI-Simple] CallMessage: {:?}", msg);

    if let Some(mut resp) = resp {
        resp.set_seqid(req.get_seqid());
        let y: u8 = 10;
        // let y: u8 = rand::thread_rng().gen();
        let body = format!("this is ReplyMessage {}", "=".repeat(y as usize));
        resp.set_body(body.as_ref());
        // eprintln!("[WASI-Simple] ReplyMessage: {:?}", msg);
    }
}
