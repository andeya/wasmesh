use wasmesh::Bytes;

use crate::http::Method;
use crate::proto::write_to_vec;

thread_local! {static AGENT: ureq::Agent = ureq::builder().build();}

pub(crate) fn do_request(req: wasmesh::Request, msg_vec: &mut Vec<u8>) -> anyhow::Result<wasmesh::Response> {
    #[cfg(debug_assertions)] { println!("got req = {:?}", req); }
    let mut builder = AGENT.with(|agent| {
        agent.request(
            Method(req.method).as_str(),
            &req.uri,
        )
    });
    for x in req.headers {
        builder = builder.set(&x.0, &x.1);
    }
    let resp = builder.send(req.body.as_ref())?;
    let resp = to_response(resp).map_err(|e| anyhow::Error::new(e))?;
    #[cfg(debug_assertions)] { println!("got resp = {:?}", resp); }
    write_to_vec(&resp, msg_vec);
    Ok(resp)
}

fn to_response(resp: ureq::Response) -> Result<wasmesh::Response, ureq::Error> {
    let mut r = wasmesh::Response::new();
    r.set_status(resp.status() as i32);
    r.set_headers(resp.headers_names().iter().map(|name| (name.clone(), resp.header(&name).unwrap().to_string())).collect());
    r.set_body(Bytes::from(resp.into_string()?));
    Ok(r)
}
