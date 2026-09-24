//! gauge-core: headless telemetry engine for the 9Gauge desktop HUD.
//!
//! Everything in this crate is GUI-free so it can be built and tested on
//! headless VPS/CI environments (`cargo test -p gauge-core`).
//! The Tauri shell (`src-tauri`, Phase 3) consumes this crate; it must never
//! contain telemetry, parsing, or auth logic itself.

pub mod auth;
pub mod error;
pub mod models;
pub mod state;
pub mod telemetry;

pub use auth::{AuthStrategy, DashboardSession, LocalCli};
pub use error::{CoreError, Result};
pub use models::UsageSnapshot;
pub use state::{AppState, ConnectionState, StateHub};
pub use telemetry::{ClientConfig, TelemetryClient};
