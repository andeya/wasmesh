/// ab（apache benchmark）
/// `ab -c 100 -n 10000 http://127.0.0.1:9090/`

#[cfg(debug_assertions)]
use std::env;

#[cfg(debug_assertions)]
use rand::Rng;
use wasp::*;

#[allow(unused_variables)]
#[wasp::handler]
fn handler(ctx_id: i64, req: Request) -> Option<Response> {
    #[cfg(debug_assertions)] {
        println!("Args: {:?}", env::args().collect::<Vec<String>>());
        println!("[WASI-Simple] Request: {:?}", req);
    }
    if req.method == Method::ONEWAY {
        return None;
    }
    #[cfg(debug_assertions)] {
        let mut req2 = Request::new();
        req2.set_method(Method::GET);
        req2.set_uri(String::from("https://github.com/henrylee2cn/wasp"));
        println!("[WASI-Simple] Request2: {:?}", req);
        let resp2 = do_request(ctx_id, req2).unwrap();
        println!("[WASI-Simple] Response2: {:?}", resp2);
    }

    let mut resp = Response::new();
    resp.set_seqid(req.seqid);
    #[cfg(not(debug_assertions))]
        let y: u8 = 10;
    #[cfg(debug_assertions)]
        let y: u8 = rand::thread_rng().gen();
    let body = format!("this is Response {}", "=".repeat(y as usize));
    resp.set_body(Bytes::from(body));
    #[cfg(debug_assertions)]
    println!("[WASI-Simple] Response: {:?}", resp);
    Some(resp)
}
