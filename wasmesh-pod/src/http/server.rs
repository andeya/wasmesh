use std::cell::RefCell;
use std::net::SocketAddr;
use std::thread::LocalKey;

use hyper::{Body, Error, Response};
use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use wasmy_vm::*;

use wasmesh_proto::*;

pub(crate) async fn serve(wasm_info: &'static LocalKey<RefCell<WasmInfo>>, addr: SocketAddr) -> anyhow::Result<()> {
    // The closure inside `make_service_fn` is run for each connection,
    // creating a 'service' to handle requests for that specific connection.
    let make_service = make_service_fn(move |_socket: &AddrStream| {
        #[cfg(debug_assertions)] {
            let remote_addr = _socket.remote_addr();
            println!("HTTP remote_addr = {:?}", remote_addr.to_string());
        }
        async move {
            // This is the `Service` that will handle the connection.
            // `service_fn` is a helper to convert a function that
            // returns a Response into a `Service`.
            Ok::<_, Error>(service_fn(move |req| async move {
                let data = HttpRequest::from(req).await;
                let r: anyhow::Result<Response<Body>> = wasm_info.with(|wi| {
                    let resp: HttpResponse = call_wasm(wi.borrow().clone(), WasmMethod::W_HTTP.into(), data)?;
                    Ok(resp.into())
                });
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
