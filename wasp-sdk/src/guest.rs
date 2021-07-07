use crate::message::Message;

// #[cfg(target_arch = "wasm32")]
extern "C" {
    /// Recall the next message data to the offset position.
    fn _wasp_host_recall_msg(thread_id: i32, ctx_id: i32, offset: i32);
    /// Send reply msg;
    fn _wasp_host_reply_msg(thread_id: i32, ctx_id: i32, offset: i32, size: i32);
    /// Send call msg, return response msg size;
    fn _wasp_host_send_msg(thread_id: i32, ctx_id: i32, offset: i32, size: i32) -> i32;
}

pub fn run_handler<F>(thread_id: i32, ctx_id: i32, size: i32, handler: F)
    where
        F: Fn(Message) -> Message,
{
    let call_msg = recv_msg(thread_id, ctx_id, size);
    let reply_msg = handler(call_msg);
    let b = serde_json::to_vec(&reply_msg).unwrap();
    unsafe { _wasp_host_reply_msg(thread_id, ctx_id, b.as_ptr() as i32, b.len() as i32) };
}

pub fn recv_msg(thread_id: i32, ctx_id: i32, size: i32) -> Message {
    if size == 0 {
        return Message::default();
    }
    let buffer = vec![0u8; size as usize];
    unsafe { _wasp_host_recall_msg(thread_id, ctx_id, buffer.as_ptr() as i32) };
    serde_json::from_slice(buffer.as_slice())
        // .unwrap()
        .unwrap_or_else(|e| {
            eprintln!("recv_msg serde_json error: {}", e);
            Message::default()
        })
}

pub fn send_msg(thread_id: i32, ctx_id: i32, msg: Message) -> Message {
    let b = serde_json::to_vec(&msg).unwrap();
    recv_msg(thread_id, ctx_id, unsafe { _wasp_host_send_msg(thread_id, ctx_id, b.as_ptr() as i32, b.len() as i32) })
}
