// Standalone test for external push functionality
// Run with: cargo run --release -p tests-ws --bin test_push
// Requires: ./target/release/nostos examples/rweb_external_push.nos

use futures_util::StreamExt;
use serde_json::Value;
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tokio_tungstenite::{connect_async, tungstenite::Message};

#[tokio::main]
async fn main() {
    println!("=== External Push Test ===");
    println!("Requires: ./target/release/nostos examples/rweb_external_push.nos");
    println!("");

    sleep(Duration::from_millis(500)).await;

    match test_external_push().await {
        Ok(_) => {
            println!("\n=== PASSED ===");
            std::process::exit(0);
        }
        Err(e) => {
            println!("\n=== FAILED: {} ===", e);
            std::process::exit(1);
        }
    }
}

async fn test_external_push() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to ws://localhost:8080/ws...");

    let (mut ws, _) = connect_async("ws://localhost:8080/ws").await?;
    println!("Connected!");

    // Receive initial full page
    println!("Waiting for initial page...");
    let msg = timeout(Duration::from_secs(5), ws.next())
        .await?
        .ok_or("Connection closed")??;

    let initial: Value = match msg {
        Message::Text(text) => serde_json::from_str(&text)?,
        other => return Err(format!("Unexpected message type: {:?}", other).into()),
    };

    assert_eq!(initial["type"], "full", "Expected initial 'full' message");
    println!("Got initial page (type=full)");

    // Send an action to the server to test two-way communication and get response
    println!("Sending action to server...");
    use futures_util::SinkExt;
    ws.send(Message::Text(r#"{"action":"increment","params":{}}"#.to_string())).await?;

    // Wait for the action response
    println!("Waiting for action response...");
    let response = timeout(Duration::from_secs(5), ws.next())
        .await?
        .ok_or("Connection closed")??;
    match response {
        Message::Text(text) => {
            let msg: Value = serde_json::from_str(&text)?;
            println!("Got action response: type={}", msg["type"]);
        }
        _ => return Err("Expected text message for action response".into()),
    }

    // Now wait for the background pusher (runs every 2 seconds)
    println!("Waiting for background push (up to 5 seconds)...");
    let push_result = timeout(Duration::from_secs(5), ws.next()).await;

    match push_result {
        Ok(Some(Ok(Message::Text(text)))) => {
            let msg: Value = serde_json::from_str(&text)?;
            println!("Received push: type={}", msg["type"]);

            let msg_type = msg["type"].as_str().unwrap_or("");
            if msg_type != "update" && msg_type != "full" {
                return Err(format!("Expected 'update' or 'full', got '{}'", msg_type).into());
            }

            if let Some(html) = msg["html"].as_str() {
                if html.contains("Push #") || html.contains("push-status") {
                    println!("Push contains expected content!");
                } else {
                    println!("Push HTML: {}...", &html[..html.len().min(100)]);
                }
            }

            Ok(())
        }
        Ok(Some(Ok(other))) => {
            Err(format!("Unexpected message type: {:?}", other).into())
        }
        Ok(Some(Err(e))) => {
            Err(format!("WebSocket error: {}", e).into())
        }
        Ok(None) => {
            Err("Connection closed before receiving push".into())
        }
        Err(_) => {
            Err("Timeout: No background push received within 5 seconds".into())
        }
    }
}
