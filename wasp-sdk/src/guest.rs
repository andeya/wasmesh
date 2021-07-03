use crate::proto::Message;

// #[cfg(target_arch = "wasm32")]
extern "C" {
    /// Return response msg size;
    fn _wasp_send_msg(offset: i32) -> i32;
    /// Return the next message size
    fn _wasp_recall_msg_size() -> i32;
    /// Recall the next message data to the offset position.
    fn _wasp_recall_msg_data(offset: i32);
}

pub fn send_msg(msg: Message) -> Message {
    let b = serde_json::to_vec(&msg).unwrap();
    recv_msg(unsafe { _wasp_send_msg(b.as_ptr() as i32) })
}

pub fn recv_msg(size: i32) -> Message {
    let buffer = vec![0u8; size as usize];
    unsafe { _wasp_recall_msg_data(buffer.as_ptr() as i32) };
    serde_json::from_slice(buffer.as_slice()).unwrap()
}

pub fn run_handler<F>(size: i32, handler: F)
    where
        F: Fn(Message) -> Message,
{
    let _ = send_msg(handler(recv_msg(size)));
}
