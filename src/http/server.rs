use std::net::SocketAddr;

use hyper::{Body, Error, Request as HttpRequest, Response as HttpResponse};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use wasp::*;

use crate::instance::local_instance_ref;
use crate::proto::write_to_with_cached_sizes;

pub(crate) async fn serve(addr: SocketAddr) -> anyhow::Result<()> {
    // The closure inside `make_service_fn` is run for each connection,
    // creating a 'service' to handle requests for that specific connection.
    let make_service = make_service_fn(|_socket: &AddrStream| {
        #[cfg(debug_assertions)] {
            let remote_addr = _socket.remote_addr();
            println!("HTTP remote_addr = {:?}", remote_addr.to_string());
        }
        async {
            // This is the `Service` that will handle the connection.
            // `service_fn` is a helper to convert a function that
            // returns a Response into a `Service`.
            Ok::<_, Error>(service_fn(|req| async {
                let r = handle(req).await;
                if let Err(ref e) = r {
                    eprintln!("{}", e)
                }
                r
            }))
        }
    });
    let srv = hyper::Server::bind(&addr).serve(make_service);
    println!("Listening on http://{}", addr);
    if let Err(e) = srv.await {
        eprintln!("SERVER error: {}", e);
    }
    Ok(())
}

async fn handle(req: HttpRequest<Body>) -> Result<HttpResponse<Body>, String> {
    // return Ok(Response::default());
    let mut req = to_request(req).await;

    let (thread_id, ins) = local_instance_ref();
    let ctx_id = ins.gen_ctx_id(thread_id);

    // TODO:
    req.set_seqid(ctx_id as i32);

    #[cfg(debug_assertions)]
    println!("thread_id:{}, ctx_id:{}", thread_id, ctx_id);
    let buffer_len = ins.use_mut_buffer(ctx_id, req.compute_size() as usize, |buffer| {
        write_to_with_cached_sizes(&req, buffer)
    });

    ins.call_guest_handler(ctx_id, buffer_len as i32);
    // println!("========= thread_id={}, ctx_id={}", thread_id, ctx_id);

    let buffer = ins.take_buffer(ctx_id).unwrap_or(vec![]);
    let resp = if buffer.len() > 0 {
        Response::parse_from_bytes(buffer.as_slice()).unwrap()
    } else {
        Response::new()
    };
    ins.try_reuse_buffer(thread_id, buffer);
    // println!("========= resp={:?}", resp);
    Ok(to_http_response(resp))
}

fn to_http_response(mut msg: wasp::Response) -> hyper::Response<hyper::Body> {
    let mut resp = hyper::Response::builder();
    for x in msg.headers.iter() {
        resp = resp.header(x.0, x.1);
    }
    if msg.status <= 0 {
        msg.status = 200
    }
    resp = resp.status(msg.status as u16);
    resp.body(hyper::Body::from(msg.body)).unwrap()
}

async fn to_request(req: hyper::Request<hyper::Body>) -> wasp::Request {
    let mut msg = wasp::Request::new();
    msg.set_uri(req.uri().to_string());
    msg.set_method(crate::http::Method::from(req.method().clone()).into());
    let (parts, body) = req.into_parts();
    let body = hyper::body::to_bytes(body).await.map_or_else(|_| wasp::Bytes::new(), |v| v);
    for x in parts.headers.iter() {
        msg.headers.insert(
            x.0.to_string(),
            x.1
             .to_str()
             .map_or_else(|_| String::new(), |s| s.to_string()),
        );
    }
    msg.set_body(body);
    msg
}
