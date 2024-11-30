# axum-server-timings

Time your axum handlers with just a method call!

`ServerTimings` allows you to instantiate one struct and automatically get a header
with all your timing data inserted, for use in browser devtools for performance testing.

The duration between calls to record is recorded automatically.

```rust
async fn handler() -> (ServerTimings, &'static str) {
    let mut timings = ServerTimings::new();
    tokio::time::sleep(Duration::from_secs_f32(0.1)).await;
    timings.record("wait", "How long the sleep took");
    tokio::time::sleep(Duration::from_secs_f32(0.5)).await;
    timings.record("wait2", "How long the second sleep took");
    (timings, "timings test")
}
```

## How do I hide this in production?

`RUSTFLAGS="--cfg hide_server_timings"`
