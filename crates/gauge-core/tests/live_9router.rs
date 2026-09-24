//! Integration tests against local live 9Router instance (localhost:20128).
//! Skips gracefully if local 9Router is not active or token not provided via NINEROUTER_TEST_TOKEN.

use gauge_core::auth::AuthStrategy;
use gauge_core::state::{ConnectionState, StateHub};
use gauge_core::telemetry::rest::fetch_stats;
use gauge_core::telemetry::sse::consume_sse_stream;
use reqwest::Client;
use std::time::Duration;

const LIVE_BASE_URL: &str = "http://127.0.0.1:20128";

#[tokio::test]
async fn test_live_rest_and_sse_ingestion() {
    let test_token = match std::env::var("NINEROUTER_TEST_TOKEN") {
        Ok(t) if !t.is_empty() => t,
        _ => {
            eprintln!("NINEROUTER_TEST_TOKEN not provided, skipping live integration test");
            return;
        }
    };

    let client = Client::builder()
        .cookie_store(true)
        .build()
        .expect("Client");

    // Probe 9Router reachability
    let probe = client
        .get(format!("{LIVE_BASE_URL}/api/usage/stats?period=today"))
        .header("x-9r-cli-token", &test_token)
        .send()
        .await;

    let resp = match probe {
        Ok(r) if r.status().is_success() => r,
        _ => {
            eprintln!(
                "Live 9Router not reachable or token rejected, skipping live integration test"
            );
            return;
        }
    };

    drop(resp);

    let strategy = AuthStrategy::RawCliToken(test_token);

    // 1. Verify REST endpoint
    let snapshot = fetch_stats(&client, &strategy, LIVE_BASE_URL, "today")
        .await
        .expect("REST fetch_stats should succeed on live 9Router");

    assert!(
        snapshot.total_requests > 0,
        "Expected total_requests > 0 on live instance"
    );
    assert!(
        !snapshot.by_provider.is_empty(),
        "Expected non-empty by_provider"
    );
    println!(
        "Live REST verified: {} requests, {} total tokens",
        snapshot.total_requests,
        snapshot.total_tokens()
    );

    // 2. Verify SSE stream
    let hub = StateHub::new();
    let sse_hub = hub.clone();
    let sse_client = client.clone();
    let sse_strategy = strategy.clone();

    let sse_handle = tokio::spawn(async move {
        let _ = consume_sse_stream(
            &sse_client,
            &sse_strategy,
            LIVE_BASE_URL,
            &sse_hub,
            Duration::from_secs(5),
        )
        .await;
    });

    // Wait up to 3s for first SSE frame to arrive
    let mut received = false;
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_millis(100)).await;
        let state = hub.load();
        if state.connection == ConnectionState::Healthy && state.snapshot.total_requests > 0 {
            received = true;
            println!(
                "Live SSE verified: stream delivered snapshot with {} requests",
                state.snapshot.total_requests
            );
            break;
        }
    }

    sse_handle.abort();

    assert!(
        received,
        "SSE stream did not deliver a snapshot within 3 seconds"
    );
}
