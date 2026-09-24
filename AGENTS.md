# AGENTS.md

## Repository Overview
`9gauge` is a lightweight, cross-platform desktop telemetry companion and HUD for 9Router, built with **Tauri v2 (Rust)** and **Svelte 5**.

---

## Commands & Verification Gates

### Build & Dev
```bash
# Frontend dependencies
pnpm install

# Run desktop app in development mode
pnpm tauri dev

# Run frontend only (Vite preview)
pnpm dev

# Build production binary
pnpm tauri build

# Headless core (testable di VPS tanpa GUI deps)
cargo test -p gauge-core
```

### Linting & Formatting
```bash
# Check Rust code (workspace-wide; core = headless-safe)
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all --check

# Check Frontend code
pnpm check
pnpm lint
```

---

## Architectural Guardrails (NON-NEGOTIABLE)

0. **Headless Core Split:** Semua logic telemetri/auth/state hidup di `crates/gauge-core` (pure Rust, testable tanpa GUI). `src-tauri` hanya shell (tray/window/IPC) yang mengonsumsi core — dilarang berisi parsing SSE atau auth logic.
1. **Zero-Flicker Pre-Warming:** Never instantiate windows on-demand on tray click. Windows must be initialized dormant (`visible: false`), pre-positioned, and unhidden in <30ms.
2. **Tabular Figures Only:** All metrics, tokens, percentages, and timestamps MUST use `JetBrains Mono` with `font-feature-settings: "tnum"`. No text wobble allowed.
3. **Suspended Webview Governance:** Popover hidden = `window.hide()` + IPC pause signal ke frontend (MVP). Deep COM suspend (`TrySuspend`) adalah optimasi P2, bukan dependensi MVP.
4. **Permissive Ingestion (Fail-Open):** Ingest telemetry from 9Router using permissive `serde` attributes (`#[serde(default)]`). An upstream provider outage must never crash the desktop daemon.
5. **Auth Reality (ADR-0003):** Endpoint telemetri 9Router menerima `x-9r-cli-token` (lokal) atau `Cookie: auth_token` JWT (remote) — BUKAN Bearer API key (terverifikasi 401). JWT 24h → silent re-login throttled.
6. **Data-Driven Providers:** UI merender `byProvider` secara dinamis; dilarang hardcode daftar provider.
7. **Tray Text Reality:** Dynamic tray text hanya di macOS; Windows/Linux memakai ikon bitmap + tooltip.
8. **Git Commit Conventions:** Follow Conventional Commits:
   - `feat(tray): add dynamic title and status dot`
   - `fix(telemetry): handle reconnect backoff on socket drop`
   - `style(ui): align hero token meter to AETER frosted glass tokens`
   - PR merge WAJIB full `--merge` or `--rebase`; DILARANG squash commit.
