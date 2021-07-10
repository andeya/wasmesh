use capnp::message::ReaderSegments;

use crate::*;

// #[cfg(target_arch = "wasm32")]
extern "C" {
    /// Recall the next request to the offset position.
    fn _wasp_recall_request(thread_id: i32, ctx_id: i32, offset: i32);
    /// Send response;
    fn _wasp_send_response(thread_id: i32, ctx_id: i32, offset: i32, size: i32);
    /// Send request, return response msg size;
    fn _wasp_send_request(thread_id: i32, ctx_id: i32, offset: i32, size: i32) -> i32;
    /// Recall the next response to the offset position.
    fn _wasp_recall_response(thread_id: i32, ctx_id: i32, offset: i32);
}

pub fn handle_request<F>(thread_id: i32, ctx_id: i32, size: i32, handler: F)
    where
        F: Fn(request::Reader, Option<response::Builder>),
{
    let mut buffer = vec![0u8; size as usize];
    unsafe { _wasp_recall_request(thread_id, ctx_id, buffer.as_ptr() as i32) };
    let reader = read_message(&mut buffer.as_slice(), ReaderOptions::new()).unwrap();
    let req = reader.get_root::<request::Reader>().unwrap();
    if req.get_oneway() {
        handler(req, None);
        return;
    }
    let mut builder = Builder::new_default();
    let mut resp = builder.init_root::<response::Builder>();
    resp.set_seqid(req.get_seqid());
    handler(req, Some(resp));
    buffer.truncate(0);
    write_message(&mut buffer, &builder).unwrap();
    unsafe {
        _wasp_send_response(thread_id, ctx_id, buffer.as_ptr() as i32, buffer.len() as i32)
    };
}

pub fn do_request<'a, F, F2>(thread_id: i32, ctx_id: i32, req_setter: F, resp_handler: Option<F2>)
    where
        F: Fn(request::Builder),
        F2: Fn(response::Reader),
{
    let mut builder = Builder::new_default();
    req_setter(builder.init_root());
    let mut buffer = Vec::with_capacity(builder.len());
    write_message(&mut buffer, &builder).unwrap();
    let size = unsafe { _wasp_send_request(thread_id, ctx_id, buffer.as_ptr() as i32, buffer.len() as i32) };
    let req: request::Reader = builder.get_root_as_reader().unwrap();
    if req.get_oneway() {
        return;
    }
    buffer.resize(size as usize, 0);
    unsafe { _wasp_recall_response(thread_id, ctx_id, buffer.as_ptr() as i32) };
    let reader = read_message(&mut buffer.as_slice(), ReaderOptions::new()).unwrap();
    if let Some(resp_handler) = resp_handler {
        resp_handler(reader.get_root::<response::Reader>().unwrap());
    }
}
