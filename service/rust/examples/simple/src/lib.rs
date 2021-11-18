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
    // let resp = handle_http(ctx_id, req);
    let resp = match req.get_scheme() {
        Ok(Scheme::HTTP | Scheme::HTTPS) => {
            handle_http(ctx_id, req)
        }
        Ok(Scheme::RPC) => { handle_rpc(ctx_id, req) }
        Ok(Scheme::WNS) => { unimplemented!() }
        Err(e) => {
            eprintln!("[WASI-Simple] get_scheme error={}", e);
            return None;
        }
    };

    #[cfg(debug_assertions)]
    println!("[WASI-Simple] Response: {:?}", resp);
    if let Err(e) = resp {
        eprintln!("[WASI-Simple] Response error={}", e);
        return None;
    }
    Some(resp.unwrap())
}

#[allow(unused_variables)]
fn handle_http(ctx_id: i64, req: Request) -> anyhow::Result<Response> {
    #[cfg(debug_assertions)] {
        let mut req2 = Request::new();
        req2.set_method(Method::GET);
        req2.set_uri(String::from("https://github.com/henrylee2cn/wasp"));
        println!("[WASI-Simple] send HTTP Request2: {:?}", req2);
        let resp2 = do_request(ctx_id, req2).unwrap();
        println!("[WASI-Simple] recv HTTP Response2: {:?}", resp2);

        let mut req3 = Request::new();
        req3.set_uri(format!("rpc://{}/test/rpc", "127.0.0.1:9091"));
        println!("[WASI-Simple] send RPC Request3: {:?}", req3);
        let resp3 = do_request(ctx_id, req3).unwrap();
        println!("[WASI-Simple] recv RPC Response3: {:?}", resp3);
    }

    let mut resp = Response::new();
    resp.set_seqid(req.seqid);
    #[cfg(not(debug_assertions))]
        let y: u8 = 10;
    #[cfg(debug_assertions)]
        let y: u8 = rand::thread_rng().gen();
    let body = format!("this is HTTP Response {}", "=".repeat(y as usize));
    resp.set_body(Bytes::from(body));

    Ok(resp)
}

#[allow(unused_variables)]
fn handle_rpc(ctx_id: i64, req: Request) -> anyhow::Result<Response> {
    let mut resp = Response::new();
    resp.set_seqid(req.seqid);
    #[cfg(not(debug_assertions))]
        let y: u8 = 10;
    #[cfg(debug_assertions)]
        let y: u8 = rand::thread_rng().gen();
    let body = format!("this is RPC Response {}", "=".repeat(y as usize));
    resp.set_body(Bytes::from(body));

    Ok(resp)
}
