//! Auth strategies for telemetry ingestion (ADR-0003).
//!
//! Live probes on running 9Router (2026-09-22):
//! - `/api/usage/*` returns **401** with Bearer API key → STRATEGY = CLI token or dashboard JWT cookie.
//! - `x-9r-cli-token` header → **200 OK**.
//! - `Cookie: auth_token=<dashboard JWT>` → **200 OK**; expiry = 24h.

use reqwest::{Client, RequestBuilder};
use secrecy::{ExposeSecret, SecretString};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::error::{CoreError, Result};

/// Local host-only strategy: derive the CLI token from the machine-id and cli-secret files in the
/// 9Router data directory. This is the recommended default when running side-by-side with the
/// local 9Router instance.
#[derive(Debug, Clone)]
pub struct LocalCli {
    /// Path to the 9Router data dir (`~/.9router/`, or `$DATA_DIR`).
    pub data_dir: PathBuf,
}

impl LocalCli {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    /// Compute the 16-hex CLI token from raw inputs (no file I/O).
    /// Formula: sha256(machine_id + "9r-cli-auth" + secret)[0..16]
    pub fn compute_token_raw(machine_id: &str, secret: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(format!("{}9r-cli-auth{}", machine_id.trim(), secret.trim()));
        let hash = hasher.finalize();
        hex::encode(&hash[..8])
    }

    /// Read machine-id and auth/cli-secret from `data_dir` and derive token.
    pub fn read_token(&self) -> Result<String> {
        let machine_id_path = self.data_dir.join("machine-id");
        let secret_path = self.data_dir.join("auth").join("cli-secret");

        let machine_id = std::fs::read_to_string(&machine_id_path).map_err(|e| {
            CoreError::Auth(format!(
                "Failed to read machine-id at {}: {}",
                machine_id_path.display(),
                e
            ))
        })?;

        let secret = std::fs::read_to_string(&secret_path).map_err(|e| {
            CoreError::Auth(format!(
                "Failed to read cli-secret at {}: {}",
                secret_path.display(),
                e
            ))
        })?;

        Ok(Self::compute_token_raw(&machine_id, &secret))
    }
}

/// Remote-host strategy: persist a dashboard password (keychain), perform an HTTP POST to login,
/// and attach the resulting JWT as a session cookie via reqwest cookie store.
/// On 401 response, silently re-login with lockout guards (max one attempt per 60s).
#[derive(Debug, Clone)]
pub struct DashboardSession {
    pub password: SecretString,
    last_login_attempt: Arc<Mutex<Option<Instant>>>,
    min_retry_interval: Duration,
    logged_in: Arc<AtomicBool>,
}

impl DashboardSession {
    pub fn new(password: SecretString) -> Self {
        Self {
            password,
            last_login_attempt: Arc::new(Mutex::new(None)),
            min_retry_interval: Duration::from_secs(60),
            logged_in: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn with_retry_interval(mut self, interval: Duration) -> Self {
        self.min_retry_interval = interval;
        self
    }

    pub fn is_logged_in(&self) -> bool {
        self.logged_in.load(Ordering::Relaxed)
    }

    /// Perform login against `/api/auth/login`. Respects per-IP lockout guards.
    pub async fn do_login(&self, base_url: &str, client: &Client) -> Result<()> {
        let mut last_attempt = self.last_login_attempt.lock().await;
        if let Some(prev) = *last_attempt {
            let elapsed = prev.elapsed();
            if elapsed < self.min_retry_interval {
                let wait = self.min_retry_interval - elapsed;
                return Err(CoreError::Auth(format!(
                    "Login throttled to protect from lockout. Retry allowed in {:.1}s",
                    wait.as_secs_f32()
                )));
            }
        }
        *last_attempt = Some(Instant::now());

        let url = format!("{}/api/auth/login", base_url.trim_end_matches('/'));
        debug!("Attempting dashboard login at {}", url);

        let body = serde_json::json!({
            "password": self.password.expose_secret()
        });

        let resp = client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(CoreError::Http)?;

        if resp.status().is_success() {
            info!("Dashboard login successful, JWT captured in cookie store");
            self.logged_in.store(true, Ordering::Relaxed);
            Ok(())
        } else {
            let status = resp.status();
            self.logged_in.store(false, Ordering::Relaxed);
            warn!("Dashboard login failed with status {}", status);
            Err(CoreError::Auth(format!("Login failed: HTTP {}", status)))
        }
    }
}

/// Unified auth strategy enum matching ADR-0003.
#[derive(Debug, Clone)]
pub enum AuthStrategy {
    Local(LocalCli),
    Remote(DashboardSession),
    /// Raw CLI token header for testing / headless integration.
    RawCliToken(String),
    /// Unauthenticated / testing mode.
    None,
}

impl AuthStrategy {
    /// Apply auth headers/cookies to an outgoing request.
    pub async fn apply(
        &self,
        client: &Client,
        base_url: &str,
        mut req: RequestBuilder,
    ) -> Result<RequestBuilder> {
        match self {
            AuthStrategy::Local(cli) => {
                let token = cli.read_token()?;
                req = req.header("x-9r-cli-token", token);
                Ok(req)
            }
            AuthStrategy::RawCliToken(token) => {
                req = req.header("x-9r-cli-token", token);
                Ok(req)
            }
            AuthStrategy::Remote(session) => {
                if !session.is_logged_in() {
                    session.do_login(base_url, client).await?;
                }
                // Reqwest cookie_store will automatically send the session cookie
                Ok(req)
            }
            AuthStrategy::None => Ok(req),
        }
    }

    /// Called when receiving 401 Unauthorized to trigger silent re-login or error.
    pub async fn on_unauthorized(&self, client: &Client, base_url: &str) -> Result<()> {
        match self {
            AuthStrategy::Remote(session) => {
                session.do_login(base_url, client).await?;
                Ok(())
            }
            AuthStrategy::Local(_) => Err(CoreError::Auth(
                "Local CLI token rejected by 9Router. Verify machine-id and cli-secret permissions.".into(),
            )),
            AuthStrategy::RawCliToken(_) => Err(CoreError::Auth(
                "Raw CLI token rejected by 9Router.".into(),
            )),
            AuthStrategy::None => Err(CoreError::Auth(
                "Endpoint requires authentication, but AuthStrategy::None was configured.".into(),
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_known_cli_token_raw() {
        let machine_id = "test-machine-id-0001";
        let secret = "test-cli-secret-key-9999";
        let token = LocalCli::compute_token_raw(machine_id, secret);
        assert_eq!(token, "e205ae867cd8e7c4");
    }

    #[test]
    fn derives_token_from_files() {
        let temp_dir = std::env::temp_dir().join(format!("9r_test_{}", std::process::id()));
        let auth_dir = temp_dir.join("auth");
        std::fs::create_dir_all(&auth_dir).unwrap();

        let machine_id = "test-machine-id-0001\n";
        let secret = "  test-cli-secret-key-9999 \n";

        std::fs::write(temp_dir.join("machine-id"), machine_id).unwrap();
        std::fs::write(auth_dir.join("cli-secret"), secret).unwrap();

        let cli = LocalCli::new(temp_dir.clone());
        let token = cli.read_token().unwrap();
        assert_eq!(token, "e205ae867cd8e7c4");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[tokio::test]
    async fn throttles_rapid_dashboard_logins() {
        let session = DashboardSession::new(SecretString::from("bad-pass".to_string()))
            .with_retry_interval(Duration::from_millis(500));
        let client = Client::new();

        // First attempt will fail connection/server (e.g. invalid host 127.0.0.1:9999), but records attempt timestamp
        let _ = session.do_login("http://127.0.0.1:9999", &client).await;

        // Immediate second attempt must be throttled
        let err = session.do_login("http://127.0.0.1:9999", &client).await;
        assert!(err.is_err());
        let err_msg = err.unwrap_err().to_string();
        assert!(
            err_msg.contains("Login throttled"),
            "Expected throttled error, got: {}",
            err_msg
        );

        // After waiting past retry interval, attempt is permitted again
        tokio::time::sleep(Duration::from_millis(550)).await;
        let err2 = session.do_login("http://127.0.0.1:9999", &client).await;
        let err2_msg = err2.unwrap_err().to_string();
        assert!(
            !err2_msg.contains("Login throttled"),
            "Expected network/http error, not throttled: {}",
            err2_msg
        );
    }
}
