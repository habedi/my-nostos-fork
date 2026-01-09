# RWeb vs Phoenix LiveView Benchmark

This document describes how to benchmark Nostos RWeb against Phoenix LiveView for reactive web application performance.

## Overview

Both frameworks use WebSocket connections for real-time UI updates:
- **Phoenix LiveView**: Elixir/Erlang BEAM-based, process-per-connection
- **Nostos RWeb**: Rust VM with green threads, process-per-session

The benchmark measures a simple counter application: click a button, increment a counter, re-render the component, send the diff.

## Test Setup

### Hardware/Environment
- Record your CPU, RAM, OS version
- Ensure no other heavy processes running
- Run server and client on same machine (localhost) to minimize network variance

### Benchmark Parameters
- **Clients**: 1000 concurrent WebSocket connections
- **Clicks per client**: 1000
- **Total operations**: 1,000,000 clicks

## Phoenix LiveView Setup

### 1. Create Phoenix Project

```bash
cd /tmp
mix phx.new liveview_bench --no-ecto --no-mailer --no-dashboard --no-gettext
cd liveview_bench
```

### 2. Create Counter LiveView

Create `lib/liveview_bench_web/live/counter_live.ex`:

```elixir
defmodule LiveviewBenchWeb.CounterLive do
  use LiveviewBenchWeb, :live_view

  def mount(_params, _session, socket) do
    {:ok, assign(socket, count: 0)}
  end

  def handle_event("inc", _params, socket) do
    {:noreply, assign(socket, count: socket.assigns.count + 1)}
  end

  def render(assigns) do
    ~H"""
    <div>
      <h1>Counter Demo</h1>
      <span>Count: <%= @count %></span>
      <button phx-click="inc">+1</button>
    </div>
    """
  end
end
```

### 3. Update Router

Edit `lib/liveview_bench_web/router.ex`:

```elixir
scope "/", LiveviewBenchWeb do
  pipe_through :browser
  live "/", CounterLive
end
```

### 4. Start Phoenix Server

```bash
mix phx.server
```

Server runs at http://localhost:4000

### 5. Phoenix Benchmark Client

Create `/tmp/liveview_bench/test_client/Cargo.toml`:

```toml
[package]
name = "lv_bench"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
futures-util = "0.3"
serde_json = "1"
reqwest = { version = "0.11", features = ["cookies"] }
rand = "0.8"
urlencoding = "2"
```

Create `/tmp/liveview_bench/test_client/src/main.rs`:

```rust
// LiveView benchmark client
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

struct LvClient {
    ws: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    topic: String,
    msg_ref: u32,
}

impl LvClient {
    async fn connect() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // First get the page to extract CSRF token and LiveView ID
        let client = reqwest::Client::builder()
            .cookie_store(true)
            .build()?;

        let page = client.get("http://localhost:4000/").send().await?.text().await?;

        // Extract csrf token
        let csrf = page
            .split("csrf-token\" content=\"")
            .nth(1)
            .and_then(|s| s.split("\"").next())
            .ok_or("No CSRF token")?;

        // Extract phx-session
        let session = page
            .split("data-phx-session=\"")
            .nth(1)
            .and_then(|s| s.split("\"").next())
            .ok_or("No session")?;

        // Extract phx-static
        let phx_static = page
            .split("data-phx-static=\"")
            .nth(1)
            .and_then(|s| s.split("\"").next())
            .unwrap_or("");

        // Extract phx-id
        let phx_id = page
            .split("id=\"phx-")
            .nth(1)
            .and_then(|s| s.split("\"").next())
            .map(|s| format!("phx-{}", s))
            .ok_or("No phx-id")?;

        // Connect to WebSocket
        let ws_url = format!(
            "ws://localhost:4000/live/websocket?_csrf_token={}&vsn=2.0.0",
            urlencoding::encode(csrf)
        );

        let (mut ws, _) = connect_async(&ws_url).await?;
        let topic = format!("lv:{}", phx_id);

        // Send join message
        let join_msg = json!([
            "1", "1", &topic, "phx_join",
            {
                "url": "http://localhost:4000/",
                "params": {"_csrf_token": csrf},
                "session": session,
                "static": phx_static
            }
        ]);

        ws.send(Message::Text(join_msg.to_string())).await?;

        // Wait for join reply
        let _msg = timeout(Duration::from_secs(5), ws.next()).await?.ok_or("closed")??;

        Ok(Self { ws, topic, msg_ref: 2 })
    }

    async fn click(&mut self) -> Result<Duration, Box<dyn std::error::Error + Send + Sync>> {
        self.msg_ref += 1;

        let event_msg = json!([
            "1", self.msg_ref.to_string(), &self.topic, "event",
            { "type": "click", "event": "inc", "value": {} }
        ]);

        let start = Instant::now();
        self.ws.send(Message::Text(event_msg.to_string())).await?;

        let _msg = timeout(Duration::from_secs(5), self.ws.next()).await?.ok_or("closed")??;
        Ok(start.elapsed())
    }

    async fn close(mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.ws.close(None).await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("LiveView Benchmark");
    println!("==================");

    let num_clients = 1000;
    let clicks_per_client = 1000;

    println!("Spawning {} clients, {} clicks each...", num_clients, clicks_per_client);

    let start = Instant::now();

    let mut handles = vec![];
    for _ in 0..num_clients {
        handles.push(tokio::spawn(async move {
            let result: Result<u128, String> = async {
                let mut client = LvClient::connect().await.map_err(|e| e.to_string())?;
                let mut total_time: u128 = 0;
                for _ in 0..clicks_per_client {
                    let elapsed = client.click().await.map_err(|e| e.to_string())?;
                    total_time += elapsed.as_millis();
                }
                client.close().await.map_err(|e| e.to_string())?;
                Ok(total_time / clicks_per_client as u128)
            }.await;
            result
        }));
    }

    let mut times: Vec<u128> = vec![];
    let mut failures = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(avg)) => times.push(avg),
            Ok(Err(e)) => { eprintln!("Error: {}", e); failures += 1; }
            Err(e) => { eprintln!("Task error: {}", e); failures += 1; }
        }
    }

    let duration = start.elapsed();
    let total_clicks = (num_clients - failures) * clicks_per_client;
    times.sort();

    if !times.is_empty() {
        println!("\nResults:");
        println!("  Total clicks: {}", total_clicks);
        println!("  Duration: {:.2}s", duration.as_secs_f64());
        println!("  Throughput: {:.0} clicks/sec", total_clicks as f64 / duration.as_secs_f64());
        println!("  Avg latency P50: {}ms", times[times.len() / 2]);
        println!("  Avg latency P99: {}ms", times[times.len() * 99 / 100]);
        println!("  Success: {}, Failures: {}", num_clients - failures, failures);
    }

    Ok(())
}
```

Build and run:
```bash
cd /tmp/liveview_bench/test_client
cargo build --release
./target/release/lv_bench
```

## Nostos RWeb Setup

### 1. Build Nostos

```bash
cd /path/to/nostos
cargo build --release
```

Optional: For maximum performance (15% faster), add LTO to `Cargo.toml`:
```toml
[profile.release]
debug = true
lto = "fat"           # +15% throughput, but 5-10 min compile time
codegen-units = 1
panic = "abort"
```

### 2. Counter Application

The example is at `examples/rweb_counter.nos`:

```nostos
use stdlib.rweb.*
use stdlib.rhtml.*

reactive Counter = { value: Int }

sessionSetup(writerId) = {
    counter = Counter(0)

    renderPage = () => RHtml(div([
        h1("RWeb Counter"),
        component("display", () => RHtml(
            div([
                span("Count: "),
                span(show(counter.value))
            ])
        )),
        div([
            button("+", dataAction: "inc"),
            button("-", dataAction: "dec"),
            button("Reset", dataAction: "reset")
        ])
    ]))

    onAction = (action, params) => match action {
        "inc" -> { counter.value = counter.value + 1 }
        "dec" -> { counter.value = counter.value - 1 }
        "reset" -> { counter.value = 0 }
        _ -> ()
    }

    (renderPage, onAction)
}

main() = startRWeb(8080, "RWeb Counter", sessionSetup)
```

### 3. Start Nostos Server

```bash
./target/release/nostos examples/rweb_counter.nos
```

Server runs at http://localhost:8080

### 4. Nostos Benchmark Client

Create `/tmp/nostos_bench/Cargo.toml`:

```toml
[package]
name = "nostos_bench"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-tungstenite = "0.21"
futures-util = "0.3"
serde_json = "1"
```

Create `/tmp/nostos_bench/src/main.rs`:

```rust
use futures_util::{SinkExt, StreamExt};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("Nostos RWeb Benchmark");
    println!("=====================");

    let num_clients = 1000;
    let clicks_per_client = 1000;

    println!("Spawning {} clients, {} clicks each...", num_clients, clicks_per_client);

    let start = Instant::now();

    let mut handles = vec![];
    for _ in 0..num_clients {
        handles.push(tokio::spawn(async move {
            let result: Result<u128, String> = async {
                let (mut ws, _) = connect_async("ws://localhost:8080/ws")
                    .await.map_err(|e| e.to_string())?;

                // Wait for initial full page message
                let _initial = timeout(Duration::from_secs(5), ws.next())
                    .await.map_err(|_| "timeout".to_string())?
                    .ok_or("closed".to_string())?
                    .map_err(|e| e.to_string())?;

                let mut total_time: u128 = 0;
                for _ in 0..clicks_per_client {
                    let start = Instant::now();
                    ws.send(Message::Text(r#"{"action":"inc","params":{}}"#.to_string()))
                        .await.map_err(|e| e.to_string())?;

                    let _resp = timeout(Duration::from_secs(5), ws.next())
                        .await.map_err(|_| "timeout".to_string())?
                        .ok_or("closed".to_string())?
                        .map_err(|e| e.to_string())?;

                    total_time += start.elapsed().as_millis();
                }

                ws.close(None).await.ok();
                Ok(total_time / clicks_per_client as u128)
            }.await;
            result
        }));
    }

    let mut times: Vec<u128> = vec![];
    let mut failures = 0;
    for handle in handles {
        match handle.await {
            Ok(Ok(avg)) => times.push(avg),
            Ok(Err(e)) => { eprintln!("Error: {}", e); failures += 1; }
            Err(e) => { eprintln!("Task error: {}", e); failures += 1; }
        }
    }

    let duration = start.elapsed();
    let total_clicks = (num_clients - failures) * clicks_per_client;
    times.sort();

    if !times.is_empty() {
        println!("\nResults:");
        println!("  Total clicks: {}", total_clicks);
        println!("  Duration: {:.2}s", duration.as_secs_f64());
        println!("  Throughput: {:.0} clicks/sec", total_clicks as f64 / duration.as_secs_f64());
        println!("  Avg latency P50: {}ms", times[times.len() / 2]);
        println!("  Avg latency P99: {}ms", times[times.len() * 99 / 100]);
        println!("  Success: {}, Failures: {}", num_clients - failures, failures);
    }

    Ok(())
}
```

Build and run:
```bash
cd /tmp/nostos_bench
cargo build --release
./target/release/nostos_bench
```

## Results

### Test Run (January 2026)

| Metric | Phoenix LiveView | Nostos RWeb |
|--------|-----------------|-------------|
| Throughput | 11,413 clicks/sec | **22,381 clicks/sec** |
| P50 Latency | 16ms | 43ms |
| P99 Latency | 16ms | 44ms |
| Success Rate | 100% | 100% |

### Optional: LTO Impact

With LTO enabled (slower compile, faster runtime):

| Metric | Default | With LTO | Improvement |
|--------|---------|----------|-------------|
| Throughput | 22,381/sec | 25,692/sec | +15% |
| P50 Latency | 43ms | 38ms | -12% |

### Analysis

**Throughput**: Nostos achieves ~2x the throughput of Phoenix LiveView (2.25x with LTO).

**Latency**: The higher latency in Nostos at 1000 concurrent clients is explained by Little's Law:
```
Latency = Concurrency / Throughput
Nostos: 1000 / 22,381 = 45ms (observed: 43ms) ✓
```

At lower concurrency (100 clients), Nostos latency drops to **3ms P50**.

**Key insight**: All Nostos processing operations (JSON parsing, action handling, component rendering, WebSocket send) complete in <1ms. The latency at scale is purely queueing time due to the cooperative scheduler.

### Protocol Differences

- **Phoenix**: Uses a complex protocol with CSRF tokens, session cookies, join handshake, topic-based routing
- **Nostos RWeb**: Simple JSON messages `{"action":"name","params":{}}`, no CSRF (session-per-connection)

## Reproducing

1. Ensure both servers are stopped before starting a benchmark
2. Start the server you want to test
3. Wait 2 seconds for warmup
4. Run the corresponding benchmark client
5. Record results
6. Stop the server

For consistent results:
- Run multiple times and average
- Use `taskset` to pin to specific CPU cores if needed
- Increase file descriptor limits: `ulimit -n 65535`
