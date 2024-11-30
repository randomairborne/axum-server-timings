use std::time::Duration;

use axum::{routing::get, Router};
use axum_server_timings::ServerTimings;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let router = Router::new().route("/", get(test_handler)).with_state(());
    let tcp = TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(tcp, router).await.unwrap();
}

async fn test_handler() -> (ServerTimings, &'static str) {
    let mut timings = ServerTimings::new();
    tokio::time::sleep(Duration::from_secs_f32(0.1)).await;
    timings.record("wait", "How long the sleep took");
    tokio::time::sleep(Duration::from_secs_f32(0.5)).await;
    timings.record("wait2", "How long the second sleep took");
    let fibn = 42;
    let fibn_suffix = ordningify(fibn);
    fib(fibn);
    timings.record("fib", format!("Find {fibn}{fibn_suffix} fibonacci number"));
    (timings, "timings test")
}

const fn fib(n: u64) -> u64 {
    match n {
        0 => 0,
        1 | 2 => 1,
        n => fib(n - 1) + fib(n - 2),
    }
}

const fn ordningify(n: u64) -> &'static str {
    match n % 10 {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    }
}
