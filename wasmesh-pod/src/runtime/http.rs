use wasmy_vm::*;

use wasmesh_proto::*;

thread_local! {static AGENT: ureq::Agent = ureq::builder().build();}

// wasmesh_pod::VmMethod::V_HTTP
#[vm_handler(0)]
fn request(req: HttpRequest) -> Result<HttpResponse> {
    #[cfg(debug_assertions)]  println!("http: got request = {:?}", req);
    let mut builder = AGENT.with(|agent| {
        agent.request(
            req.get_method().as_str(),
            req.get_url(),
        )
    });
    for (header, value) in req.get_headers() {
        builder = builder.set(header, value);
    }
    let resp = builder.send(req.body.as_ref()).map_err(|e| ERR_CODE_UNKNOWN.to_code_msg(e))?;
    let mut r = HttpResponse::new();
    r.set_status(resp.status() as i32);
    r.set_headers(resp.headers_names().iter().map(|name| (name.clone(), resp.header(&name).unwrap().to_string())).collect());
    r.set_body(Bytes::from(resp.into_string()?));
    #[cfg(debug_assertions)]  println!("http: got response = {:?}", r);
    Ok(r)
}
