# Tasks Execution Plan: 9Gauge

## Status: Ready for Execution (GATE 1 Review — revised after live audit 2026-09-22)
**Target:** Monorepo Tauri v2 + Rust Core + Svelte 5 Frontend
**Workspace topology:** `crates/gauge-core` (headless, testable di VPS) + `src-tauri` (GUI shell) + `src/` (Svelte 5). Lihat `docs/ARCHITECTURE.md` §0.

---

## Phase 1 — Headless Core & Telemetry Engine (Rust, no GUI deps)

### Task 1: Cargo Workspace & Core Scaffolding
- [x] Initialize workspace: `Cargo.toml` (`members = ["crates/*", "src-tauri"]`) + `crates/gauge-core`.
- [x] Core deps: `tokio`, `reqwest` (cookies, json, stream), `eventsource-stream`, `serde`, `serde_json`, `arc-swap`, `compact_str`, `secrecy`, `thiserror`, `tracing`.
- [x] `models.rs`: fail-open serde contracts — `UsageSnapshot`, `ProviderEntry`, `RecentRequest`, `ActiveRequest` — semua `#[serde(default)]`, verifikasi terhadap fixture payload live (termasuk `last10Minutes`, `pending`, `byApiKey`, `byEndpoint`).
- **Verification:** `cargo test -p gauge-core` lulus di VPS headless (fixture = payload live yang di-capture).

### Task 2: Auth Strategies (ADR-0003) — Local CLI Token & Dashboard Session
- [x] `AuthStrategy::LocalCli` — baca `machine-id` + `auth/cli-secret` dari data dir 9Router, derive `sha256(raw + "9r-cli-auth" + secret)[0..16]`, kirim header `x-9r-cli-token`.
- [x] `AuthStrategy::DashboardSession` — `POST /api/auth/login` dengan password keychain → cache JWT in-memory → header `Cookie: auth_token=...`.
- [x] Silent re-login pada 401 (throttle ≥60s, lockout-safe); JWT TIDAK pernah disimpan ke disk.
- [x] DILARANG mengirim Bearer API key ke endpoint telemetri (terverifikasi 401).
- **Verification:** Integration test ke live 9Router (localhost:20128) → 200 di kedua strategi; mock 401 → re-login tunggal.

### Task 3: Async Telemetry Ingestion & State Pipeline
- [x] `telemetry/sse.rs` — consume `/api/usage/stream` via `eventsource-stream`; heartbeat watchdog 35s (`: ping` aktual 25s).
- [x] `telemetry/rest.rs` — fallback/poller `/api/usage/stats?period=` (`today|24h|7d|30d|60d|all`).
- [x] Reconnect FSM: `Connecting → Healthy → Degraded → Disconnected → Reconnecting`; jittered backoff 1s→30s.
- [x] `state.rs` — `ArcSwap<AppState>` + `RecentRequestsRingBuffer<20>` (CompactStr).
- [x] TANPA debouncer IPC (audit: server sudah throttle 150–250ms); TANPA deep COM suspend (MVP = hide + IPC pause signal; deep suspend P2).
- **Verification:** `cargo test -p gauge-core` (mock SSE server di test) + integration test melawan live stream.

---

## Phase 2 — AETER Frosted Glass Frontend (Tasks 4–6)

### Task 4: Design Tokens & Layout Shell
- [x] Implement CSS variables and Tailwind utilities for AETER frosted glass (`rgba(13, 15, 20, 0.82)` and `rgba(255, 255, 255, 0.78)`), double-bezel specular highlights, and backdrop blur.
- [x] Import `Plus Jakarta Sans` and `JetBrains Mono` fonts with `font-feature-settings: "tnum"`.
- [x] Build Header component: Host selector breadcrumb (`✦ 9GAUGE / TELEMETRY`), period dropdown (`Today ▾`), and status breathing dot.
- **Verification:** Popover matches visual specification with zero layout shift.

### Task 5: Hero Token Meter & Segmented Visualizer
- [x] Build `HeroTokenMeter.svelte` displaying total consumed tokens with smooth tabular counting.
- [x] Build segmented progress bar showing Prompt vs. Completion vs. Cached tokens with tooltips.
- [x] Surface token velocity dari `last10Minutes[]` (bukan perhitungan klien) dan request throughput (`req/min`).
- **Verification:** Numbers render cleanly with `tabular-nums` without font shaking during rapid count updates.

### Task 6: Provider Cards & Real-Time Micro-Feed (DATA-DRIVEN — audit revised)
- [x] Build `ProviderList.svelte` rendering cards **dinamis dari key `byProvider`** — DILARANG hardcode daftar provider (live: `antigravity`, `qoder`, `openai-compatible-chat-<uuid>`).
- [x] Surface balance / reset cycle / health pill bila metadata kuota tersedia (periode-agnostik untuk provider tanpa kuota).
- [x] Build `MicroFeed.svelte` displaying recent requests dengan timestamp, model, latency, dan status code.
- [x] Build `RTKSavingsBadge.svelte` showing tokens compressed by 9Router.
- **Verification:** Correctly parses and displays multi-provider usage from live 9Router telemetry snapshot.

---

## Phase 3 — Tauri Shell, Sentinel Alerting & Packaging (Tasks 7–9)

### Task 7: Tauri Shell Integration, Tray Adapter & Host Switcher
- [x] `src-tauri` mengonsumsi `gauge-core` (tanpa logic telemetri di shell).
- [x] Tray adapter per-platform: macOS = dynamic text title + dot; Windows/Linux = ikon bitmap status + tooltip (DILARANG tray text — audit).
- [x] Popover: pre-warmed, non-activating (macOS NSPanel + global mouse-down monitor untuk dismissal — `Focused(false)` hanya fallback/Win/Linux).
- [x] Implement Settings flyout untuk switch host Localhost ↔ Remote VPS; remote auth = dashboard password di OS keychain + silent re-login (ADR-0003).
- [x] Native OS notification untuk HTTP 429 dan low balance (< $2.00).
- **Verification:** Mock 429 → toast OS + dot Rose; switch host → auth flow sesuai strategi masing-masing.

### Task 8: CI/CD Multi-Platform Build Pipeline
- [x] Create `.github/workflows/release.yml` with matrix: macOS (Universal DMG), Windows (NSIS + MSI), Linux (deb + AppImage).
- [x] CI job headless: `cargo fmt --check && cargo clippy -D warnings && cargo test` pada `crates/gauge-core` (tanpa GUI deps).
- [x] Author final release notes and binary signing configuration.
- **Verification:** GitHub Actions runner compiles and outputs artifacts successfully.
