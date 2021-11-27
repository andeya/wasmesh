use std::thread;
use std::time::Duration;

use structopt::{clap::AppSettings, StructOpt};
use wasmesh::*;

use wasmesh_pod::request;

#[derive(StructOpt, Debug, Clone)]
#[structopt(global_settings = & [AppSettings::VersionlessSubcommands, AppSettings::ColorAuto, AppSettings::ColoredHelp])]
struct BenchArgs {
    /// Number of requests to perform
    #[structopt(long, short = "n", default_value = "10000")]
    requests: u64,
    /// Number of multiple requests to make at a time
    #[structopt(long, short = "c", default_value = "10")]
    concurrency: u64,
    /// HTTP/RPC URI: [http|rpc]://hostname:port/path
    uri: String,
}

fn main() {
    let args: BenchArgs = BenchArgs::from_args();
    println!("{:?}", args);
    let mut tasks = vec![];
    for _ in 0..args.concurrency {
        let args_clone = args.clone();
        let task = thread::spawn(move || {
            let mut fail_count = 0u64;
            let t = std::time::Instant::now();
            for _ in 0..args_clone.requests {
                if !do_request(&args_clone) {
                    fail_count += 1;
                }
            }
            (fail_count, t.elapsed())
        });
        tasks.push(task);
    }

    let mut fail_count = 0u64;
    let mut cast = Duration::new(0, 0);
    for x in tasks {
        let r = x.join().unwrap();
        fail_count += r.0;
        if r.1 > cast {
            cast = r.1;
        }
    }
    let total_requests = args.requests * args.concurrency;

    println!("Concurrency Level:\t{:}", args.concurrency);
    println!("Time taken for tests:\t{:.3} seconds", cast.as_secs_f64());
    println!("Complete requests:\t{}", total_requests);
    println!("Failed requests:\t{}", fail_count);
    println!("Requests per second:\t{:.3} [#/sec] (mean)", total_requests as f64 / cast.as_secs_f64());
}

fn do_request(args: &BenchArgs) -> bool {
    let mut req = Request::new();
    req.set_uri(args.uri.clone());
    request(req)
        .map_or_else(|e| {
            eprintln!("{}", e);
            false
        }, |_resp| {
            // println!("{:?}", resp);
            true
        })
}
