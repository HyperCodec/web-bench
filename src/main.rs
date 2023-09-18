use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use clap::Parser;
use env_logger::Env;
use url::Url;

use log::{debug, error, info};
use reqwest::blocking::{Client, Request};
use reqwest::Method;

#[derive(Parser, Debug, Clone)]
#[command(
    name = "wb",
    author = "HyperCodec",
    about = "A simple tool to bench server connections"
)]
struct Cli {
    #[clap(short, long, help = "The server you want to ping")]
    url: Url,

    #[clap(
        short,
        long,
        help = "How long you want to ping the server for",
        default_value = "30"
    )]
    secs: u64,

    #[clap(
        short,
        long,
        help = "How many threads you want to use",
        default_value = "1"
    )]
    thread_count: usize,
}

#[derive(Debug)]
struct BenchResult {
    req_count: u64,
    unsuccessful_count: u64,
}

fn main() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let args = Cli::parse();
    let dur = Duration::from_secs(args.secs);

    let results = bench(args);

    info!("Finished benchmark");

    let req_count = results.iter().map(|r| r.req_count).sum::<u64>();
    let unsuccessful_count = results.iter().map(|r| r.unsuccessful_count).sum::<u64>();

    info!("Requests sent: {req_count}");

    let successful_count = req_count - unsuccessful_count;
    info!("Successful requests: {successful_count}");

    info!("Unsuccessful requests: {unsuccessful_count}");

    let sr = successful_count as f64 / req_count as f64;
    info!("Success ratio: {sr:.2}");

    let rps = req_count as f64 / dur.as_secs_f64();
    info!("Requests per second: {rps:.2}");

    let mpr = dur.as_millis() as f64 / req_count as f64;
    info!("MS per request: {mpr:.2}");
}

fn bench(args: Cli) -> Vec<BenchResult> {
    debug!("Setting up channel and stuff");
    let (tx, rx) = mpsc::channel::<BenchResult>();
    let mut handles = Vec::with_capacity(args.thread_count);

    info!("Benchmarking server");

    // spawn tasks
    for _ in 0..args.thread_count {
        let args = args.clone();
        let tx = tx.clone();

        let h = thread::spawn(move || {
            let client = Client::new();
            let req = Request::new(Method::HEAD, args.url);

            let mut req_count = 0;
            let mut unsuccessful_count = 0;

            let dur = Duration::from_secs(args.secs);

            let start = Instant::now();

            // spam
            while start.elapsed() < dur {
                debug!("Sending request");

                let res = client.execute(req.try_clone().unwrap());

                match res {
                    Ok(res) => {
                        debug!("Received response");
                        let s = res.status();
                        if !s.is_success() {
                            debug!("Unsuccessful response ({})", s);
                            unsuccessful_count += 1;
                        }
                    }
                    Err(err) => {
                        error!("Error sending request: {}", err);
                        unsuccessful_count += 1;
                    }
                }

                req_count += 1;
            }

            // send yummy result
            tx.send(BenchResult {
                req_count,
                unsuccessful_count,
            })
            .expect("Failed to send results across channel");
        });

        handles.push(h);
    }

    let mut out = Vec::with_capacity(args.thread_count);

    // wait for tasks to finish and send yummy results
    debug!("Receiving messages from channel");
    for _ in 0..args.thread_count {
        out.push(rx.recv().expect("Channel closed for some reason"));
    }

    debug!("Waiting on threads to end");
    for h in handles {
        h.join().expect("Failed to join thread");
    }

    out
}
