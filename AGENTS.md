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
```

### Linting & Formatting
```bash
# Check Rust code
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path src-tauri/Cargo.toml --check

# Check Frontend code
pnpm check
pnpm lint
```

---

## Architectural Guardrails (NON-NEGOTIABLE)

1. **Zero-Flicker Pre-Warming:** Never instantiate windows on-demand on tray click. Windows must be initialized dormant (`visible: false`), pre-positioned, and unhidden in <30ms.
2. **Tabular Figures Only:** All metrics, tokens, percentages, and timestamps MUST use `JetBrains Mono` with `font-feature-settings: "tnum"`. No text wobble allowed.
3. **Suspended Webview Governance:** Explicitly suspend the Webview runtime when the popover is hidden to ensure idle RAM stays <18MB and CPU stays 0.0%.
4. **Permissive Ingestion (Fail-Open):** Ingest telemetry from 9Router using permissive `serde` attributes (`#[serde(default)]`). An upstream provider outage must never crash the desktop daemon.
5. **Git Commit Conventions:** Follow Conventional Commits:
   - `feat(tray): add dynamic title and status dot`
   - `fix(telemetry): handle reconnect backoff on socket drop`
   - `style(ui): align hero token meter to AETER frosted glass tokens`
   - PR merge WAJIB full `--merge` or `--rebase`; DILARANG squash commit.
