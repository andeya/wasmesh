use crate::message::{FlatBufferBuilder, Message, MessageArgs, MessageBuilder, RefMessage};

// #[cfg(target_arch = "wasm32")]
extern "C" {
    /// Recall the next message data to the offset position.
    fn _wasp_host_recall_msg(thread_id: i32, ctx_id: i32, offset: i32);
    /// Send reply msg;
    fn _wasp_host_reply_msg(thread_id: i32, ctx_id: i32, offset: i32, size: i32);
    /// Send call msg, return response msg size;
    fn _wasp_host_send_msg(thread_id: i32, ctx_id: i32, offset: i32, size: i32) -> i32;
}

pub fn run_handler<'a, F>(thread_id: i32, ctx_id: i32, size: i32, handler: F)
    where
        F: Fn(&Message, &mut MessageBuilder),
{
    let call_msg = recv_msg(thread_id, ctx_id, size);
    let mut builder = FlatBufferBuilder::new();
    let mut reply_builder = MessageBuilder::new(&mut builder);
    handler(&call_msg.as_ref(), &mut reply_builder);
    let offset = reply_builder.finish();
    builder.finish_minimal(offset);
    let reply_msg_bytes = builder.finished_data();
    unsafe { _wasp_host_reply_msg(thread_id, ctx_id, reply_msg_bytes.as_ptr() as i32, reply_msg_bytes.len() as i32) };
}

pub fn recv_msg(thread_id: i32, ctx_id: i32, size: i32) -> RefMessage {
    let buffer = vec![0u8; size as usize];
    unsafe { _wasp_host_recall_msg(thread_id, ctx_id, buffer.as_ptr() as i32) };
    RefMessage::from_vec(buffer)
}

pub fn send_msg(thread_id: i32, ctx_id: i32, msg_args: &MessageArgs) -> RefMessage {
    let mut builder = FlatBufferBuilder::new();
    let offset = Message::create(&mut builder, msg_args);
    builder.finish_minimal(offset);
    let msg_args_bytes = builder.finished_data();
    recv_msg(thread_id, ctx_id, unsafe { _wasp_host_send_msg(thread_id, ctx_id, msg_args_bytes.as_ptr() as i32, msg_args_bytes.len() as i32) })
}

pub fn to_bytes(msg_args: &MessageArgs) -> (Vec<u8>, usize) {
    let mut builder = FlatBufferBuilder::new();
    let offset = Message::create(&mut builder, msg_args);
    builder.finish_minimal(offset);
    builder.collapse()
}
