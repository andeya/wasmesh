use wasp_sdk::proto::{RequestData, ResponseData};

fn main() {
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let req = RequestData::from_reader(stdin.lock());
    match req {
        Ok(req) => {
            if let Err(e) = handle(req).to_writer(stdout.lock()) {
                eprintln!("[WASI-Simple] {}", e);
            }
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
