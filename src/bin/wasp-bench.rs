use structopt::{clap::AppSettings, StructOpt};
use wasp::*;

use wasp_cli::request;

#[derive(StructOpt, Debug)]
#[structopt(global_settings = & [AppSettings::VersionlessSubcommands, AppSettings::ColorAuto, AppSettings::ColoredHelp])]
struct BenchArgs {
    /// Number of requests to perform
    #[structopt(long, short = "n", default_value = "1000")]
    requests: u64,
    /// Number of multiple requests to make at a time
    #[structopt(long, short = "c", default_value = "1")]
    concurrency: u32,
    /// HTTP/RPC URI: [http|rpc]://hostname:port/path
    uri: String,
}

#[tokio::main]
async fn main() {
    let args: BenchArgs = BenchArgs::from_args();
    println!("{:?}", args);
    let t = std::time::Instant::now();
    for _ in 0..args.requests {
        do_request(&args);
    }
    let cast = t.elapsed();
    let qps = args.requests as f32 / cast.as_secs_f32();
    println!("Requests per second:\t{:.3} [#/sec] (mean)", qps);
    println!("Time taken for tests:\t{:.3} seconds", cast.as_secs_f32());
}

fn do_request(args: &BenchArgs) -> bool {
    let mut req = Request::new();
    req.set_uri(args.uri.clone());
    request(req)
        .map(|_| true)
        .unwrap_or(false)
}
