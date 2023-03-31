# Shutdown Async

[<img alt="github" src="https://img.shields.io/badge/github-wcygan/shutdown--async-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/wcygan/shutdown-async)
[<img alt="crates.io" src="https://img.shields.io/crates/v/shutdown-async.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/shutdown-async)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-shutdown--async-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/shutdown-async)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/wcygan/shutdown-async/test.yml?branch=main&style=for-the-badge" height="20">](https://github.com/wcygan/shutdown-async/actions?query=branch%3Amain)
[![codecov](https://codecov.io/gh/wcygan/shutdown-async/branch/main/graph/badge.svg?token=AGLZ67JA6D)](https://codecov.io/gh/wcygan/shutdown-async)

A library for gracefully shutting down asynchronous applications

This may be useful when you want to allow all in-flight processing to complete before shutting down in order to maintain a consistent state.

# Usage

Add this to your Cargo.toml:

```toml
[dependencies]
shutdown-async = "0.1.1"
```

You can use the library like so:

```rust
use shutdown_async::ShutdownController;

#[tokio::main]
async fn main() {
  let shutdown = ShutdownController::new();
   
  tokio::task::spawn({
    let mut monitor = shutdown.subscribe();
    async move {
      // Wait for something to happen
      tokio::select! {
       _ = monitor.recv() => { println!("shutdown initiated"); }
       _ = tokio::time::sleep(ONE_YEAR) => { println!("one year has passed!"); }
      }
    }
  });

  shutdown.shutdown().await;
}

static ONE_YEAR: std::time::Duration = std::time::Duration::from_secs(60 * 60 * 24 * 365);
```