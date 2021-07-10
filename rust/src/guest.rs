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
        F: Fn(Request) -> Option<Response>,
{
    let req = recv_request(thread_id, ctx_id, size);
    let resp = handler(req);
    if let Some(resp) = resp {
        let b = &resp.write_to_bytes().unwrap();
        unsafe { _wasp_send_response(thread_id, ctx_id, b.as_ptr() as i32, b.len() as i32) };
    }
}

fn recv_request(thread_id: i32, ctx_id: i32, size: i32) -> Request {
    if size == 0 {
        return Request::default();
    }
    let buffer = vec![0u8; size as usize];
    unsafe { _wasp_recall_request(thread_id, ctx_id, buffer.as_ptr() as i32) };
    Request::parse_from_bytes(buffer.as_slice())
        .unwrap_or_else(|e| {
            eprintln!("receive request parse_from_bytes error: {}", e);
            Request::default()
        })
}

pub fn do_request(thread_id: i32, ctx_id: i32, req: Request) -> Option<Response> {
    let mut buffer = req.write_to_bytes().unwrap();
    let size = unsafe { _wasp_send_request(thread_id, ctx_id, buffer.as_ptr() as i32, buffer.len() as i32) };
    if size <= 0 || req.get_oneway() {
        return None
    }
    buffer.resize(size as usize, 0);
    unsafe { _wasp_recall_response(thread_id, ctx_id, buffer.as_ptr() as i32) };
    Some(Response::parse_from_bytes(buffer.as_slice())
        .unwrap_or_else(|e| {
            eprintln!("receive response parse_from_bytes error: {}", e);
            Response::default()
        }))
}
