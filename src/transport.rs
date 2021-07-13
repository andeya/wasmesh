use wasp::*;

use crate::instance;
use crate::proto::ServeOpt;

// trait Transport {
//     async fn serve(serve_options: ServeOpt) -> anyhow::Result<()>;
//     fn do_request(req: wasp::Request) -> anyhow::Result<wasp::Response>;
// }

pub(crate) fn serve(serve_options: ServeOpt) -> anyhow::Result<()> {
    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder.worker_threads(serve_options.get_worker_threads());
    builder.enable_all()
           .build()
           .unwrap()
           .block_on(async {
               instance::rebuild(&serve_options).await.unwrap();
               let (http_res, ) = tokio::join!(
                   crate::http::serve(serve_options),
                   // TODO: RPC
               );
               http_res
           })
}

pub(crate) fn do_request(req: wasp::Request) -> anyhow::Result<wasp::Response> {
    match req.get_scheme()? {
        Scheme::HTTP | Scheme::HTTPS => crate::http::do_request(req),
        Scheme::RPC => unimplemented!(),
        Scheme::WNS => unimplemented!(),
    }
}
