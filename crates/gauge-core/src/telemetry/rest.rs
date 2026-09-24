//! REST telemetry fallback & stats polling (ADR-0003).

use reqwest::Client;
use tracing::{debug, warn};

use crate::auth::AuthStrategy;
use crate::error::{CoreError, Result};
use crate::models::UsageSnapshot;

/// Fetch snapshot from `/api/usage/stats?period=<period>`.
/// Valid periods in 9Router: "today", "24h", "7d", "30d", "60d", "all".
pub async fn fetch_stats(
    client: &Client,
    strategy: &AuthStrategy,
    base_url: &str,
    period: &str,
) -> Result<UsageSnapshot> {
    let clean_url = format!(
        "{}/api/usage/stats?period={}",
        base_url.trim_end_matches('/'),
        period
    );
    debug!("Fetching telemetry stats from {}", clean_url);

    let req = client.get(&clean_url);
    let req = strategy.apply(client, base_url, req).await?;
    let resp = req.send().await.map_err(CoreError::Http)?;

    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        warn!("Received 401 Unauthorized while fetching stats; invoking auth retry");
        strategy.on_unauthorized(client, base_url).await?;

        // Retry request once after re-authenticating
        let req = client.get(&clean_url);
        let req = strategy.apply(client, base_url, req).await?;
        let retry_resp = req.send().await.map_err(CoreError::Http)?;

        if !retry_resp.status().is_success() {
            let status = retry_resp.status();
            return Err(CoreError::Auth(format!(
                "HTTP {} after re-authenticating",
                status
            )));
        }
        let snapshot = retry_resp
            .json::<UsageSnapshot>()
            .await
            .map_err(CoreError::Http)?;
        return Ok(snapshot);
    }

    if !resp.status().is_success() {
        return Err(CoreError::Http(resp.error_for_status().unwrap_err()));
    }

    let snapshot = resp
        .json::<UsageSnapshot>()
        .await
        .map_err(CoreError::Http)?;
    Ok(snapshot)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    #[tokio::test]
    async fn fetches_stats_from_mock_endpoint() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let base_url = format!("http://127.0.0.1:{port}");

        tokio::spawn(async move {
            if let Ok((mut socket, _)) = listener.accept().await {
                let mut buf = [0u8; 1024];
                let _ = socket.read(&mut buf).await;

                let body = r#"{"totalRequests": 99, "totalPromptTokens": 8888}"#;
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = socket.write_all(response.as_bytes()).await;
                let _ = socket.flush().await;
            }
        });

        let client = Client::new();
        let strategy = AuthStrategy::None;

        let snap = fetch_stats(&client, &strategy, &base_url, "today")
            .await
            .unwrap();

        assert_eq!(snap.total_requests, 99);
        assert_eq!(snap.total_prompt_tokens, 8888);
    }
}
