# Tasks Execution Plan: 9Gauge

## Status: Ready for Execution (GATE 1 Review)
**Target:** Monorepo Tauri v2 + Rust Core + Svelte 5 Frontend  

---

## Phase 1 — Core Scaffolding & Native Desktop Host (Tasks 1–3)

### Task 1: Scaffolding & Dependency Matrix
- [ ] Initialize Tauri v2 project structure (`src-tauri` + frontend root with Svelte 5).
- [ ] Configure `Cargo.toml` with `tauri`, `tokio`, `reqwest`, `eventsource-stream`, `serde`, `serde_json`, `arc-swap`, `compact_str`.
- [ ] Configure `package.json` with Svelte 5, Tailwind CSS, `@tauri-apps/api`, Lucide icons.
- **Verification:** `cargo check` and `pnpm build` pass with exit code 0.

### Task 2: Native Tray, Window Pre-Warming & Suspension Governance
- [ ] Implement `src-tauri/src/tray.rs` utilizing `TrayIconBuilder`.
- [ ] Configure dynamic tray title and custom status dot rendering.
- [ ] Implement non-activating popover window positioning with `tauri-plugin-positioner`.
- [ ] Implement auto-hide on `WindowEvent::Focused(false)`.
- [ ] Implement background Webview suspension hooks (`TrySuspend` on Windows / alpha loop detach on macOS).
- **Verification:** Tray icon appears, left-click opens popover without stealing terminal focus, blur dismisses cleanly.

### Task 3: Async Telemetry Ingestion & State Pipeline
- [ ] Implement `src-tauri/src/telemetry/client.rs` using Tokio async SSE (`/api/usage/stream`).
- [ ] Implement exponential backoff reconnect (`1s` to `30s`) and watchdog heartbeat check.
- [ ] Implement lock-free `ArcSwap<AppState>` and `RecentRequestsRingBuffer<20>` in `src-tauri/src/telemetry/state.rs`.
- [ ] Implement 20Hz adaptive IPC event debouncer to Webview.
- **Verification:** Simulated SSE stream pushes events through Rust core and emits to Webview under 1ms.

---

## Phase 2 — AETER Frosted Glass Frontend (Tasks 4–6)

### Task 4: Design Tokens & Layout Shell
- [ ] Implement CSS variables and Tailwind utilities for AETER frosted glass (`rgba(13, 15, 20, 0.82)` and `rgba(255, 255, 255, 0.78)`), double-bezel specular highlights, and backdrop blur.
- [ ] Import `Plus Jakarta Sans` and `JetBrains Mono` fonts with `font-feature-settings: "tnum"`.
- [ ] Build Header component: Host selector breadcrumb (`✦ 9GAUGE / TELEMETRY`), period dropdown (`Today ▾`), and status breathing dot.
- **Verification:** Popover matches visual specification with zero layout shift.

### Task 5: Hero Token Meter & Segmented Visualizer
- [ ] Build `HeroTokenMeter.svelte` displaying total consumed tokens with smooth tabular counting.
- [ ] Build segmented progress bar showing Prompt vs. Completion vs. Cached tokens with tooltips.
- [ ] Surface token velocity (`tokens/hour`) and request throughput (`req/min`).
- **Verification:** Numbers render cleanly with `tabular-nums` without font shaking during rapid count updates.

### Task 6: Provider Cards & Real-Time Micro-Feed
- [ ] Build `ProviderList.svelte` rendering cards for Google AG, OpenRouter, Kiro, and DeepSeek.
- [ ] Surface balance (`$14.20 left`), reset cycle countdown, and health pill.
- [ ] Build `MicroFeed.svelte` displaying the last 3 requests with timestamp, model, latency, and status code.
- [ ] Build `RTKSavingsBadge.svelte` showing tokens compressed by 9Router.
- **Verification:** Correctly parses and displays multi-provider usage from 9Router telemetry snapshot.

---

## Phase 3 — Sentinel Alerting & Packaging (Tasks 7–8)

### Task 7: Sentinel Alerts & Host Switcher
- [ ] Implement native OS notification triggers for HTTP 429 rate limits and low balance (< $2.00).
- [ ] Implement Settings flyout to switch host between Localhost (`localhost:20128`) and Remote VPS (`https://9router.aeter.my.id`) with Bearer token persistence in OS keychain.
- **Verification:** Triggering a mock 429 emits native OS toast notification and updates status dot to Rose.

### Task 8: CI/CD Multi-Platform Build Pipeline
- [ ] Create `.github/workflows/release.yml` with matrix:
  - macOS (Universal Apple Silicon + Intel DMG)
  - Windows (NSIS `.exe` + `.msi`)
  - Linux (`.deb` + `.AppImage`)
- [ ] Author final release notes and binary signing configuration.
- **Verification:** GitHub Actions runner compiles and outputs artifacts successfully.
