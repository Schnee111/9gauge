//! Telemetry orchestration client managing SSE ingestion, REST fallback, and reconnect FSM.

use reqwest::Client;
use std::time::Duration;
use tokio::sync::watch;
use tracing::{debug, info, warn};

use crate::auth::AuthStrategy;
use crate::state::{AppState, ConnectionState, StateHub};
use crate::telemetry::rest::fetch_stats;
use crate::telemetry::sse::{consume_sse_stream, DEFAULT_HEARTBEAT_TIMEOUT};

/// Configuration for the telemetry client.
#[derive(Debug, Clone)]
pub struct ClientConfig {
    /// Base URL of 9Router (e.g. `http://localhost:20128`).
    pub base_url: String,
    /// Authentication strategy (LocalCli, DashboardSession, or None).
    pub auth: AuthStrategy,
    /// Fallback REST polling interval.
    pub rest_poll_interval: Duration,
    /// SSE heartbeat watchdog timeout.
    pub sse_heartbeat_timeout: Duration,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:20128".to_string(),
            auth: AuthStrategy::None,
            rest_poll_interval: Duration::from_secs(30),
            sse_heartbeat_timeout: DEFAULT_HEARTBEAT_TIMEOUT,
        }
    }
}

/// Telemetry runner that orchestrates real-time SSE ingestion with REST fallback and
/// a self-healing reconnect state machine.
pub struct TelemetryClient {
    config: ClientConfig,
    client: Client,
    hub: StateHub,
}

impl TelemetryClient {
    pub fn new(config: ClientConfig, hub: StateHub) -> Self {
        let client = Client::builder()
            .cookie_store(true)
            .build()
            .expect("Failed to build reqwest Client");

        Self {
            config,
            client,
            hub,
        }
    }

    /// Access reference to the shared StateHub.
    pub fn hub(&self) -> &StateHub {
        &self.hub
    }

    /// Run the telemetry pipeline with graceful shutdown support.
    pub async fn run(self, mut shutdown_rx: watch::Receiver<bool>) {
        info!("Starting telemetry client for {}", self.config.base_url);
        self.hub.set_connection(ConnectionState::Connecting);

        let host_tag = self
            .config
            .base_url
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .to_string();

        // 1. Initial warm-up via REST so UI has instantaneous data before SSE establishes
        match fetch_stats(
            &self.client,
            &self.config.auth,
            &self.config.base_url,
            "today",
        )
        .await
        {
            Ok(snapshot) => {
                debug!("Initial telemetry snapshot loaded via REST");
                self.hub.publish(AppState {
                    snapshot,
                    connection: ConnectionState::Healthy,
                    host: host_tag.clone().into(),
                });
            }
            Err(e) => {
                warn!("Initial REST snapshot failed (will connect via SSE): {}", e);
            }
        }

        let mut backoff_attempt: u32 = 0;

        loop {
            if *shutdown_rx.borrow() {
                info!("Telemetry client received shutdown signal");
                self.hub.set_connection(ConnectionState::Disconnected);
                break;
            }

            self.hub.set_connection(ConnectionState::Connecting);
            debug!(
                "Attempting SSE stream connection (attempt {})",
                backoff_attempt + 1
            );

            let sse_result = consume_sse_stream(
                &self.client,
                &self.config.auth,
                &self.config.base_url,
                &self.hub,
                self.config.sse_heartbeat_timeout,
            )
            .await;

            match sse_result {
                Ok(()) => {
                    info!("SSE stream closed normally; reconnecting");
                    backoff_attempt = 0;
                    self.hub.set_connection(ConnectionState::Reconnecting);
                }
                Err(e) => {
                    warn!("SSE stream disconnected or failed: {}", e);
                    backoff_attempt = backoff_attempt.saturating_add(1);

                    // Check if error is authentication rejection
                    if e.to_string().contains("Unauthorized")
                        || e.to_string().contains("Login failed")
                    {
                        self.hub.set_connection(ConnectionState::AuthFailed);
                    } else {
                        self.hub.set_connection(ConnectionState::Degraded);
                    }

                    // Attempt REST fallback while SSE is degraded
                    if let Ok(snap) = fetch_stats(
                        &self.client,
                        &self.config.auth,
                        &self.config.base_url,
                        "today",
                    )
                    .await
                    {
                        debug!("REST fallback successfully fetched snapshot during degradation");
                        self.hub.publish(AppState {
                            snapshot: snap,
                            connection: ConnectionState::Degraded,
                            host: host_tag.clone().into(),
                        });
                    }
                }
            }

            // Exponential backoff: 1s, 2s, 4s, 8s, 16s, up to 30s + small jitter
            let backoff_secs = (1u64 << backoff_attempt.min(5)).min(30);
            // Simple pseudo-jitter derived from attempt count to avoid extra dependency
            let jitter_millis = (backoff_attempt * 137) % 500;
            let wait_duration = Duration::from_millis((backoff_secs * 1000) + jitter_millis as u64);

            debug!("Reconnecting in {:?}", wait_duration);
            self.hub.set_connection(ConnectionState::Reconnecting);

            tokio::select! {
                _ = tokio::time::sleep(wait_duration) => {}
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!("Shutdown requested during reconnect backoff");
                        self.hub.set_connection(ConnectionState::Disconnected);
                        break;
                    }
                }
            }
        }
    }
}
