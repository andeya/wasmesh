use std::io::Write;

use wasp_sdk::proto::{RequestData, ResponseData};

#[no_mangle]
fn _wasp_serve() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let req = RequestData::from_reader(stdin.lock());
    match req {
        Ok(req) => {
            let mut w = stdout.lock();
            if let Err(e) = handle(req).to_writer(&mut w) {
                eprintln!("[WASI-Simple] {}", e);
            }
            let _ = w.flush();
        }
        Err(e) => eprintln!("[WASI-Simple] {}", e),
    }
}

fn handle(req: RequestData) -> ResponseData {
    eprintln!("[WASI-Simple] RequestData: {:?}", req);
    let body = "this is ResponseData".as_bytes().to_vec();
    let resp = ResponseData::from_request_data(req, body);
    eprintln!("[WASI-Simple] ResponseData: {:?}", resp);
    resp
}
