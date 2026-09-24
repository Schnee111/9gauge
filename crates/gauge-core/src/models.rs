//! Fail-open serde contracts for 9Router telemetry payloads.
//!
//! Every field defaults — a schema change or partial payload from 9Router
//! must NEVER crash the desktop daemon (AGENTS.md guardrail #4).
//! Contract verified against live payload on 2026-09-22 (see docs/ARCHITECTURE.md §3).

use serde::{Deserialize, Deserializer};
use std::collections::BTreeMap;

/// Tolerant deserializer that turns explicit `null` in JSON into `Default::default()`.
pub fn deserialize_null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    let opt = Option::deserialize(deserializer)?;
    Ok(opt.unwrap_or_default())
}

/// Per-slice counters used across `byProvider`, `byModel`, `byAccount`, ...
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct CounterEntry {
    pub requests: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cached_tokens: u64,
    pub cost: f64,
    /// Aggregation metadata (rawModel, provider, accountName, ...) — opaque to core.
    #[serde(flatten)]
    pub meta: BTreeMap<String, serde_json::Value>,
}

/// A finished request passing through 9Router (ring buffer / feed item).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct RecentRequest {
    pub timestamp: String,
    pub model: String,
    pub provider: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cached_tokens: u64,
    pub status: String,
}

/// An in-flight request (server-side pending map).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ActiveRequest {
    pub model: String,
    pub provider: String,
    pub account: String,
    pub count: u64,
}

/// One per-minute throughput bucket from the server (`last10Minutes`).
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ThroughputBucket {
    pub requests: u64,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub cost: f64,
}

/// Full snapshot as emitted by `GET /api/usage/stats?period=...` and the
/// `data:` frames of `GET /api/usage/stream`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct UsageSnapshot {
    pub total_requests: u64,
    pub total_prompt_tokens: u64,
    pub total_completion_tokens: u64,
    pub total_cached_tokens: u64,
    pub total_cost: f64,

    /// Provider names are DYNAMIC keys (antigravity, qoder,
    /// openai-compatible-chat-<uuid>, ...). Never hardcode in UI.
    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub by_provider: BTreeMap<String, CounterEntry>,

    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub by_model: BTreeMap<String, CounterEntry>,

    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub by_account: BTreeMap<String, CounterEntry>,

    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub last_10_minutes: Vec<ThroughputBucket>,

    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub active_requests: Vec<ActiveRequest>,

    #[serde(default, deserialize_with = "deserialize_null_default")]
    pub recent_requests: Vec<RecentRequest>,

    /// Provider whose request errored within the last 10s ("" = none).
    pub error_provider: String,
}

impl UsageSnapshot {
    /// Sum of prompt + completion tokens (the hero counter's headline number).
    pub fn total_tokens(&self) -> u64 {
        self.total_prompt_tokens + self.total_completion_tokens
    }

    /// Latest per-minute prompt+completion throughput, or 0 when idle.
    pub fn current_burn_per_min(&self) -> u64 {
        self.last_10_minutes
            .last()
            .map(|b| b.prompt_tokens + b.completion_tokens)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fixture captured from the live 9Router (trimmed to contract-relevant
    /// fields, incl. the dynamic `openai-compatible-chat-<uuid>` provider).
    const LIVE_PAYLOAD: &str = r#"{
        "totalRequests": 144898,
        "totalPromptTokens": 16813439549,
        "totalCompletionTokens": 80131034,
        "totalCachedTokens": 8925056095,
        "totalCost": 17601.295,
        "byProvider": {
            "antigravity": { "requests": 1801, "promptTokens": 230418450, "completionTokens": 906289, "cachedTokens": 199137681, "cost": 95.11 },
            "openai-compatible-chat-d27df55a-127c-46a7-a87f-eeed145da7bb": { "requests": 4036, "promptTokens": 536813516, "completionTokens": 2989144, "cachedTokens": 0, "cost": 274.41 }
        },
        "byModel": {
            "gfmodel (qoder)": { "requests": 14, "promptTokens": 1757364, "completionTokens": 4898, "cachedTokens": 1336576, "cost": 0, "rawModel": "gfmodel", "provider": "qoder" }
        },
        "last10Minutes": [
            { "requests": 45, "promptTokens": 4809576, "completionTokens": 18363, "cost": 1.79 },
            { "requests": 28, "promptTokens": 4350225, "completionTokens": 7933, "cost": 1.51 }
        ],
        "activeRequests": [
            { "model": "gemini-3.8-flash-high", "provider": "antigravity", "account": "kurehaa48@gmail.com", "count": 1 }
        ],
        "recentRequests": [
            { "timestamp": "2026-09-22T02:55:56.068Z", "model": "gfmodel", "provider": "qoder", "promptTokens": 58227, "completionTokens": 96, "cachedTokens": 55680, "status": "ok" }
        ],
        "errorProvider": "antigravity"
    }"#;

    #[test]
    fn parses_live_payload() {
        let snap: UsageSnapshot =
            serde_json::from_str(LIVE_PAYLOAD).expect("live payload must parse");
        assert_eq!(snap.total_requests, 144_898);
        assert_eq!(snap.total_prompt_tokens, 16_813_439_549);
        assert_eq!(snap.by_provider.len(), 2);
        // Dynamic uuid-ish provider key survives round-trip
        assert!(snap
            .by_provider
            .contains_key("openai-compatible-chat-d27df55a-127c-46a7-a87f-eeed145da7bb"));
        assert_eq!(snap.by_provider["antigravity"].requests, 1801);
        assert_eq!(snap.by_provider["antigravity"].prompt_tokens, 230_418_450);
        assert_eq!(snap.last_10_minutes.len(), 2);
        assert_eq!(snap.active_requests[0].account, "kurehaa48@gmail.com");
        assert_eq!(snap.error_provider, "antigravity");
    }

    /// Fail-open on null values. `pending` field in original payload is a nested object with null
    /// maps; the contract must survive that.
    #[test]
    fn fail_open_on_partial_and_null_map() {
        let empty: UsageSnapshot = serde_json::from_str("{}").expect("empty object parses");
        assert_eq!(empty.total_tokens(), 0);

        // Simulate provider map being explicitly null — still parseable due to tolerant deserializer.
        let partial: UsageSnapshot =
            serde_json::from_str(r#"{"totalPromptTokens": 42, "byProvider": null}"#)
                .expect("partial payload parses");
        assert_eq!(partial.total_prompt_tokens, 42);
        assert!(partial.by_provider.is_empty());
    }

    #[test]
    fn derived_metrics() {
        let snap: UsageSnapshot = serde_json::from_str(LIVE_PAYLOAD).unwrap();
        assert_eq!(snap.total_tokens(), 16_813_439_549 + 80_131_034);
        assert_eq!(snap.current_burn_per_min(), 4_350_225 + 7_933);
    }
}
