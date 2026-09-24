//! Server-Sent Events (SSE) telemetry ingestion pipeline with 35s watchdog.

use eventsource_stream::Eventsource;
use futures::{StreamExt, TryStreamExt};
use reqwest::Client;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

use crate::auth::AuthStrategy;
use crate::error::{CoreError, Result};
use crate::models::UsageSnapshot;
use crate::state::{AppState, ConnectionState, StateHub};

/// Default watchdog timeout. 9Router sends `: ping` every ~25s.
pub const DEFAULT_HEARTBEAT_TIMEOUT: Duration = Duration::from_secs(35);

/// Ingest real-time telemetry from `/api/usage/stream` until connection drops,
/// watchdog triggers, or error occurs.
pub async fn consume_sse_stream(
    client: &Client,
    strategy: &AuthStrategy,
    base_url: &str,
    hub: &StateHub,
    heartbeat_timeout: Duration,
) -> Result<()> {
    let stream_url = format!("{}/api/usage/stream", base_url.trim_end_matches('/'));
    debug!("Connecting to SSE stream at {}", stream_url);

    let req = client.get(&stream_url);
    let req = strategy.apply(client, base_url, req).await?;
    let resp = req.send().await.map_err(CoreError::Http)?;

    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        warn!("Received 401 on SSE stream; notifying auth strategy");
        strategy.on_unauthorized(client, base_url).await?;
        return Err(CoreError::Auth("401 Unauthorized on SSE stream".into()));
    }

    if !resp.status().is_success() {
        return Err(CoreError::Http(resp.error_for_status().unwrap_err()));
    }

    info!("SSE connection established to {}", stream_url);
    hub.set_connection(ConnectionState::Healthy);

    let last_activity = Arc::new(Mutex::new(Instant::now()));
    let last_act_chunk = Arc::clone(&last_activity);

    // Track every received raw byte chunk (including `: ping` comments) to reset watchdog
    let byte_stream = resp.bytes_stream().inspect_ok(move |_| {
        if let Ok(mut l) = last_act_chunk.lock() {
            *l = Instant::now();
        }
    });

    let mut event_stream = byte_stream.eventsource();
    let host_tag = base_url
        .trim_start_matches("http://")
        .trim_start_matches("https://");

    let check_interval = Duration::from_millis(50);

    loop {
        tokio::select! {
            _ = tokio::time::sleep(check_interval) => {
                let elapsed = match last_activity.lock() {
                    Ok(guard) => guard.elapsed(),
                    Err(_) => Duration::ZERO,
                };
                if elapsed > heartbeat_timeout {
                    warn!("SSE watchdog timed out ({:.1}s without ping or event)", elapsed.as_secs_f32());
                    return Err(CoreError::Sse(format!(
                        "SSE watchdog timeout ({:.1}s without activity)",
                        elapsed.as_secs_f32()
                    )));
                }
            }
            item = event_stream.next() => {
                match item {
                    Some(Ok(event)) => {
                        let data = event.data.trim();
                        if data.is_empty() || data == "::" {
                            continue;
                        }

                        match serde_json::from_str::<UsageSnapshot>(data) {
                            Ok(snapshot) => {
                                hub.publish(AppState {
                                    snapshot,
                                    connection: ConnectionState::Healthy,
                                    host: host_tag.into(),
                                });
                            }
                            Err(e) => {
                                warn!("Failed to parse SSE payload (fail-open): {}", e);
                            }
                        }
                    }
                    Some(Err(e)) => {
                        return Err(CoreError::Sse(e.to_string()));
                    }
                    None => {
                        info!("SSE stream closed gracefully by server");
                        return Ok(());
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn consumes_mock_sse_and_updates_hub() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let base_url = format!("http://127.0.0.1:{port}");

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;

                let response_headers = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n";
                let _ = socket.write_all(response_headers.as_bytes()).await;

                // Send comment (ping) and data event
                let ping = ": ping\n\n";
                let event = "data: {\"totalRequests\": 42, \"totalPromptTokens\": 12345}\n\n";
                let _ = socket.write_all(ping.as_bytes()).await;
                let _ = socket.write_all(event.as_bytes()).await;
                let _ = socket.flush().await;

                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        });

        let client = Client::new();
        let hub = StateHub::new();
        let strategy = AuthStrategy::None;

        let res =
            consume_sse_stream(&client, &strategy, &base_url, &hub, Duration::from_secs(5)).await;

        assert!(res.is_ok());
        let current = hub.load();
        assert_eq!(current.snapshot.total_requests, 42);
        assert_eq!(current.snapshot.total_prompt_tokens, 12345);
        assert_eq!(current.connection, ConnectionState::Healthy);
    }

    #[tokio::test]
    async fn watchdog_triggers_on_silence() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let base_url = format!("http://127.0.0.1:{port}");

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;

                let response_headers = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: keep-alive\r\n\r\n";
                let _ = socket.write_all(response_headers.as_bytes()).await;
                let _ = socket.flush().await;

                // Stay silent for 300ms
                tokio::time::sleep(Duration::from_millis(300)).await;
            }
        });

        let client = Client::new();
        let hub = StateHub::new();
        let strategy = AuthStrategy::None;

        // Set watchdog timeout strictly to 100ms
        let res = consume_sse_stream(
            &client,
            &strategy,
            &base_url,
            &hub,
            Duration::from_millis(100),
        )
        .await;

        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("watchdog timeout"), "Got: {err_msg}");
    }
}
