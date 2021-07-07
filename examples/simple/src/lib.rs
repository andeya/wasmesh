/// ab（apache benchmark）
/// `ab -c 100 -n 10000 http://127.0.0.1:9090/`

use std::env;

use rand::Rng;

use wasp_sdk::message::{FlatBufferBuilder, Message, MessageArgs, MessageBuilder, MessageType};

#[wasp_sdk::handler]
fn handler(msg: &Message, reply_builder: &mut MessageBuilder) {
    // eprintln!("Args: {:?}", env::args().collect::<Vec<String>>());
    // eprintln!("[WASI-Simple] CallMessage: {:?}", msg);
    // let mut rng = rand::thread_rng();
    // let y: u8 = rng.gen();
    let y: u8 = 10;
    let mut builder = FlatBufferBuilder::new();
    let body = builder.create_vector(
        format!("this is ReplyMessage {}", "=".repeat(y as usize)).as_bytes()
    );
    reply_builder.add_seqid(msg.seqid());
    reply_builder.add_mtype(MessageType::Reply);
    reply_builder.add_body(body);
}
