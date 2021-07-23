use structopt::{clap::AppSettings, StructOpt};
use wasp::*;

use wasp_cli::request;

#[derive(StructOpt, Debug, Clone)]
#[structopt(global_settings = & [AppSettings::VersionlessSubcommands, AppSettings::ColorAuto, AppSettings::ColoredHelp])]
struct BenchArgs {
    /// Number of requests to perform
    #[structopt(long, short = "n", default_value = "1000")]
    requests: u64,
    /// Number of multiple requests to make at a time
    #[structopt(long, short = "c", default_value = "1")]
    concurrency: u64,
    /// HTTP/RPC URI: [http|rpc]://hostname:port/path
    uri: String,
}

#[tokio::main]
async fn main() {
    let args: BenchArgs = BenchArgs::from_args();
    println!("{:?}", args);
    let t = std::time::Instant::now();
    let mut tasks = vec![];
    for _ in 0..args.concurrency {
        let args_clone = args.clone();
        let task = tokio::spawn(async move {
            let mut fail_count = 0u64;
            for _ in 0..args_clone.requests {
                if !do_request(&args_clone) {
                    fail_count += 1;
                }
            }
            fail_count
        });
        tasks.push(task);
    }

    let mut fail_count = 0u64;
    for x in tasks {
        fail_count += x.await.unwrap();
    }
    let cast = t.elapsed();
    let qps = args.requests as f64 / cast.as_secs_f64();

    println!("Concurrency Level:\t{:}", args.concurrency);
    println!("Time taken for tests:\t{:.3} seconds", cast.as_secs_f64());
    println!("Complete requests:\t{}", args.requests * args.concurrency);
    println!("Failed requests:\t{}", fail_count);
    println!("Requests per second:\t{:.3} [#/sec] (mean)", qps);
}

fn do_request(args: &BenchArgs) -> bool {
    let mut req = Request::new();
    req.set_uri(args.uri.clone());
    request(req)
        .map_or_else(|e| {
            eprintln!("{}", e);
            false
        }, |_| true)
}
