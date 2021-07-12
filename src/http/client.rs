use std::convert::{TryFrom, TryInto};

use attohttpc::{Response, Result};
use attohttpc::header::HeaderName;

use crate::http::Method;

pub(crate) fn do_request(req: wasp::Request) -> anyhow::Result<wasp::Response> {
    println!("got req = {:?}", req);
    let mut builder = attohttpc::RequestBuilder::new(Method(req.method).try_into().unwrap(), req.uri);
    for x in req.headers {
        builder = builder.header(HeaderName::try_from(&x.0).unwrap(), x.1);
    }
    let resp = builder.bytes(req.body)
                      .send().unwrap();
    let resp = to_response(resp).map_err(|e| anyhow::Error::new(e));
    println!("got resp = {:?}", resp);
    resp
}

fn to_response(resp: Response) -> Result<wasp::Response> {
    let mut r = wasp::Response::new();
    r.set_status(resp.status().as_u16() as i32);
    r.set_headers(resp.headers().iter().map(|kv| (kv.0.to_string(), kv.1.to_str().unwrap().to_string())).collect());
    r.set_body(resp.bytes().unwrap().into());
    Ok(r)
}
