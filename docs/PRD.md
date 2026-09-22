# Product Requirements Document (PRD)

# PRD.md changelog
- **2026-09-22 (GATE 1 Audit):** Remote auth koreksi — Bearer API key **tidak** diterima di `/api/usage/*` (terverifikasi 401 live); gunakan CLI token (lokal) atau dashboard session cookie JWT (remote) + silent re-login tiap 24 jam (lihat `docs/ADR/0003-telemetry-auth-strategy.md`).

## Project: 9Gauge (9Router Desktop HUD & Telemetry Companion)
**Version:** 1.0.0-draft  
**Status:** In-Review (GATE 1)  
**Author:** Shorekeeper (Architect) & Schnee (Lead)  

---

## 1. Problem Statement & Motivation
Developers running AI coding agents (Claude Code, Hermes Agent, Cursor, OpenCode) through a local or remote 9Router gateway generate high-frequency LLM requests. However:
1. **Lack of Ambient Awareness:** Developers cannot see their current token burn rate, input vs. output vs. cached breakdown, or active request throughput without keeping a full browser tab open to `http://localhost:20128/dashboard`.
2. **Invisible Rate-Limit & Balance Cliff:** Upstream providers (OpenRouter, DeepSeek, Kiro, Codex) silently deplete or hit HTTP 429 rate limits mid-session, causing agent loops and confusing latency spikes.
3. **Bloat of Existing Solutions:** Existing monitoring tools either consume excessive RAM (>100MB Electron/Next.js) or lack deep token telemetry and real-time event streaming.

**9Gauge** solves this by providing a featherlight (<15MB RAM, 0.0% idle CPU) cross-platform desktop menu-bar/tray HUD that surfaces real-time token telemetry and upstream quota limits at a single glance.

---

## 2. Target Personas
* **The AI-Native Developer / Power User:** Runs coding agents all day, needs to know token spend and cache efficiency instantly.
* **The Multi-Account Solo Hacker:** Manages multiple upstream provider accounts (Google OAuth pools, OpenRouter, Kiro) and needs immediate alert when an account is throttled or depleted.
* **The Minimalist Engineer:** Demands zero background battery drain, sub-30ms popup responsiveness, and clean utilitarian aesthetics (AETER Monitor Archetype).

---

## 3. Scope & Feature Matrix

### P0 — Core MVP (Must Ship)
- **Multi-OS Native Tray:** Seamless tray icon on macOS (Menu Bar), Windows (System Tray), and Linux (AppIndicator).
- **Dynamic Tray Title:** Displays rolling token count and health breathing dot (`✦ 9R · 1.2M · $0.42 ●`).
- **Zero-Flicker Pre-Warmed Popover (<30ms):** Opens instantly on tray click, dismisses on blur (`Focused(false)`).
- **Unified Event-Driven Ingestion:** Consumes 9Router SSE stream (`/api/usage/stream`) + cached aggregations (`/api/usage/stats?period=today`).
- **Hero Token Meter:** Visualizes Total Consumed Tokens broken into:
  - Prompt Tokens (Input)
  - Completion Tokens (Output)
  - Cached Tokens (Cache Hits — Anthropic/Gemini caching)
- **Upstream Provider Breakdown (Data-Driven):** Per-provider usage, percentage share, and remaining quota/balance, rendered **dynamically dari key `byProvider`** pada payload stats. *(Koreksi audit: nama provider bersifat dinamis — live data memuat `antigravity`, `qoder`, `openai-compatible-chat-<uuid>` — dilarang hardcode daftar provider.)*
- **Auto-Reconnect & Offline State:** Exponential backoff reconnect with graceful UI degradation if 9Router restarts.

### P1 — High-Signal Enhancements
- **RTK Token-Killer Gauge:** Highlights total tokens compressed and saved by 9Router's RTK pre-translate filters.
- **Native OS Toast Notifications:** Triggers native OS alert on HTTP 429 rate limit or low balance (< $2.00 / < 15% quota).
- **Dual-Instance Host Switcher:** Instant toggle between `Localhost (http://127.0.0.1:20128)` and `Remote VPS (https://9router.aeter.my.id)`. *(Auth koreksi: remote memakai dashboard password via keychain + session cookie JWT + silent re-login 24h — BUKAN Bearer API key; lihat `docs/ADR/0003-telemetry-auth-strategy.md`.)*
- **Period Filter:** Quick toggle for `Today`, `24h`, `7d`, `All`.

### P2 — Polish & Telemetry
- **Micro-Throughput Sparkline:** 10-minute token velocity curve.
- **One-Click Quick Actions:** Copy local endpoint URL (`http://localhost:20128/v1`), open full web dashboard, force credential refresh.

---

## 4. Non-Functional Requirements (NFRs)
- **Memory Footprint:** 
  - Idle Tray Mode: **< 18 MB RSS RAM**.
  - Active Popover Open: **< 45 MB RSS RAM**.
- **CPU / Power Consumption:**
  - Idle: **0.0% CPU**, zero GPU wakeups.
  - Active Stream Ingestion: < 1% CPU during high-throughput bursts (100 req/min).
  - Webview background suspension via OS-level deep sleep hooks when window is hidden.
- **Latency & Responsiveness:**
  - Tray Click to Visible Popover: **< 30 ms**.
  - In-memory state update to UI render: **< 16 ms (60 FPS)**.
- **Visual Design Standard:** 100% adherence to the **AETER Monitor Archetype** (frosted glass 72%–80%, double-bezel, Plus Jakarta Sans + JetBrains Mono tabular numbers, no neon slop).

---

## 5. Success Metrics
- Zero UI freezing or dropped frames during 20 concurrent AI tool calls.
- Binary size < 12 MB across all desktop platforms.
- Battery impact classified as "Negligible / Low" in macOS Activity Monitor.
