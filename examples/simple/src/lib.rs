/// ab（apache benchmark）
/// `ab -c 100 -n 10000 http://127.0.0.1:9090/`

use std::env;

use rand::Rng;

use wasp_sdk::*;

#[wasp_sdk::handler]
fn handler(mut msg: Message) -> Message {
    // eprintln!("Args: {:?}", env::args().collect::<Vec<String>>());
    // eprintln!("[WASI-Simple] CallMessage: {:?}", msg);
    // let mut rng = rand::thread_rng();
    // let y: u8 = rng.gen();
    let y: u8 = 10;
    let body = format!("this is ReplyMessage {}", "=".repeat(y as usize));
    msg.set_body(Bytes::from(body));
    // eprintln!("[WASI-Simple] ReplyMessage: {:?}", msg);
    msg
}
