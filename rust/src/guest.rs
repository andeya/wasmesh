use crate::*;

// #[cfg(target_arch = "wasm32")]
extern "C" {
    /// Recall the next request to the offset position.
    fn _wasp_recall_request(ctx_id: i64, offset: i32);
    /// Send response;
    fn _wasp_send_response(ctx_id: i64, offset: i32, size: i32);
    /// Send request, return response msg size;
    fn _wasp_send_request(ctx_id: i64, offset: i32, size: i32) -> i32;
    /// Recall the next response to the offset position.
    fn _wasp_recall_response(ctx_id: i64, offset: i32);
}

pub fn handle_request<F>(ctx_id: i64, size: i32, handler: F)
    where
        F: Fn(i64, Request) -> Option<Response>,
{
    let mut buffer = vec![0u8; size as usize];
    let req = {
        if size == 0 {
            Request::default()
        } else {
            unsafe { _wasp_recall_request(ctx_id, buffer.as_ptr() as i32) };
            Request::parse_from_bytes(buffer.as_slice())
                .unwrap_or_else(|e| {
                    eprintln!("receive request parse_from_bytes error: {}", e);
                    Request::default()
                })
        }
    };
    // let req = recv_request( ctx_id, size);
    let resp = handler(ctx_id, req);
    if let Some(resp) = resp {
        let size = resp.compute_size() as usize;
        if size > buffer.capacity() {
            buffer.resize(size, 0);
        }
        unsafe { buffer.set_len(size) };
        let mut os = CodedOutputStream::bytes(&mut buffer);
        resp.write_to_with_cached_sizes(&mut os)
            .or_else(|e| Err(format!("{}", e))).unwrap();
        unsafe { _wasp_send_response(ctx_id, buffer.as_ptr() as i32, buffer.len() as i32) };
    }
}

// fn recv_request( ctx_id: i64, size: i32) -> Request {
//     if size == 0 {
//         return Request::default();
//     }
//     let buffer = vec![0u8; size as usize];
//     unsafe { _wasp_recall_request( ctx_id, buffer.as_ptr() as i32) };
//     Request::parse_from_bytes(buffer.as_slice())
//         .unwrap_or_else(|e| {
//             eprintln!("receive request parse_from_bytes error: {}", e);
//             Request::default()
//         })
// }

pub fn do_request(ctx_id: i64, req: Request) -> Option<Response> {
    let mut buffer = req.write_to_bytes().unwrap();
    let size = unsafe { _wasp_send_request(ctx_id, buffer.as_ptr() as i32, buffer.len() as i32) };
    if size <= 0 || req.get_method() == Method::ONEWAY {
        return None
    }
    buffer.resize(size as usize, 0);
    unsafe { _wasp_recall_response(ctx_id, buffer.as_ptr() as i32) };
    Some(Response::parse_from_bytes(buffer.as_slice())
        .unwrap_or_else(|e| {
            eprintln!("receive response parse_from_bytes error: {}", e);
            Response::default()
        }))
}
