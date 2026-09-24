//! Lock-free state hub shared between the telemetry engine (writer) and the
//! Tauri shell / IPC layer (readers). ArcSwap gives atomic lock-free reads;
//! writers clone-and-swap so ingestion never blocks the UI thread.

use arc_swap::ArcSwap;
use compact_str::CompactString;
use std::sync::Arc;

use crate::models::UsageSnapshot;

/// Capacity of the recent-requests ring (mirrors the 20-item deduped window
/// on the server; recentRequests payload is capped at 20 there).
pub const RECENT_RING_CAP: usize = 20;

/// Fixed-capacity, zero-allocation-after-warmup ring of recent requests.
#[derive(Debug)]
pub struct RecentRing {
    slots: Vec<crate::models::RecentRequest>,
    head: usize,
    len: usize,
}

impl RecentRing {
    pub fn new(cap: usize) -> Self {
        Self {
            slots: Vec::with_capacity(cap),
            head: 0,
            len: 0,
        }
    }

    pub fn push(&mut self, item: crate::models::RecentRequest) {
        if self.slots.len() < self.slots.capacity() {
            self.slots.push(item);
            self.len += 1;
            return;
        }
        self.slots[self.head] = item;
        self.head = (self.head + 1) % self.slots.len();
    }

    /// Newest-first iteration.
    pub fn iter_newest_first(&self) -> impl Iterator<Item = &crate::models::RecentRequest> {
        let cap = self.slots.capacity();
        let is_full = self.slots.len() == cap;
        let head = self.head;
        let len = self.len;

        (0..len).map(move |i| {
            let idx = if is_full {
                (head + cap * 2 - 1 - (i % cap)) % cap
            } else {
                len - 1 - i
            };
            &self.slots[idx]
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
}

/// Connection health of the telemetry engine (drives tray dot + UI banner).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    /// Connecting for the first time.
    #[default]
    Connecting,
    /// Streaming normally.
    Healthy,
    /// Connected but data is stale / partial (degraded heartbeat).
    Degraded,
    /// Manually or gracefully disconnected.
    Disconnected,
    /// Socket dropped, backoff in progress.
    Reconnecting,
    /// Auth rejected and re-login exhausted / unavailable.
    AuthFailed,
}

/// Immutable state handed to the UI on every swap.
#[derive(Debug, Clone, Default)]
pub struct AppState {
    pub snapshot: UsageSnapshot,
    pub connection: ConnectionState,
    /// Host the state belongs to (e.g. "localhost:20128").
    pub host: CompactString,
}

/// Shared hub: engine writes via [`StateHub::publish`], UI reads via [`StateHub::load`].
#[derive(Debug, Clone, Default)]
pub struct StateHub {
    inner: Arc<ArcSwap<AppState>>,
}

impl StateHub {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(ArcSwap::new(Arc::new(AppState::default()))),
        }
    }

    pub fn publish(&self, state: AppState) {
        self.inner.store(Arc::new(state));
    }

    pub fn load(&self) -> Arc<AppState> {
        self.inner.load_full()
    }

    pub fn set_connection(&self, connection: ConnectionState) {
        self.inner.rcu(|prev| {
            let mut next = (**prev).clone();
            next.connection = connection;
            next
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RecentRequest;

    #[test]
    fn ring_wraps_and_keeps_newest_first() {
        let mut ring = RecentRing::new(3);
        for i in 0..5u64 {
            ring.push(RecentRequest {
                model: format!("m{i}"),
                ..Default::default()
            });
        }
        assert_eq!(ring.len(), 3);
        let models: Vec<String> = ring
            .iter_newest_first()
            .map(|r| r.model.to_string())
            .collect();
        // newest first: the 3 most recent pushes (m4, m3, m2)
        assert_eq!(
            models,
            vec!["m4", "m3", "m2"],
            "ring should yield newest items first"
        );
    }

    #[test]
    fn ring_under_capacity() {
        let mut ring = RecentRing::new(20);
        ring.push(RecentRequest::default());
        assert_eq!(ring.len(), 1);
        assert_eq!(ring.iter_newest_first().count(), 1);
    }

    #[test]
    fn hub_swap_and_connection_update() {
        let hub = StateHub::new();
        assert_eq!(hub.load().connection, ConnectionState::Connecting);

        hub.publish(AppState {
            snapshot: UsageSnapshot {
                total_requests: 7,
                ..Default::default()
            },
            connection: ConnectionState::Healthy,
            host: CompactString::from("localhost:20128"),
        });
        let state = hub.load();
        assert_eq!(state.snapshot.total_requests, 7);
        assert_eq!(state.host.as_str(), "localhost:20128");

        hub.set_connection(ConnectionState::Reconnecting);
        let state = hub.load();
        assert_eq!(state.connection, ConnectionState::Reconnecting);
        // rcu preserved the snapshot
        assert_eq!(state.snapshot.total_requests, 7);
    }
}
