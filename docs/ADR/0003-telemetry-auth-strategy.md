# ADR 0003: Multi-Strategy Telemetry Auth (CLI Token Local, JWT Cookie Remote, No Bearer API Key)

## Status
Accepted — verified empirically against live 9Router instance (2026-09-22).

## Context
The original plan assumed 9Gauge could authenticate to both local and remote 9Router instances using a Bearer API key persisted in the OS keychain. Empirical verification against the running 9Router (`src/dashboardGuard.js`, live curl probes) disproved this:

| Probe | Result |
|---|---|
| `GET /api/usage/stats` unauthenticated | **401** |
| `GET /api/usage/stats` + `Authorization: Bearer <api key>` | **401** — API keys are only honored on the public LLM surface (`/v1`, `/api/v1`, ...), never on `PROTECTED_API_PATHS` |
| `GET /api/usage/stats` + `Cookie: auth_token=<dashboard JWT>` | **200 OK** — full stats payload |
| `GET /api/usage/stream` + dashboard JWT cookie | **200 OK** — SSE events flow |
| `GET /api/usage/stats` + `x-9r-cli-token: <derived>` | **200 OK** |

Auth gates in `dashboardGuard.js` for `/api/usage/*`:
1. `x-9r-cli-token` header — derived locally as `sha256(machineId + "9r-cli-auth" + cliSecret)[0..16]`, where `machineId` and `cliSecret` are 0600 files under the 9Router data dir (`~/.9router/`, or `$DATA_DIR` in Docker). Readable only by processes on the same host/filesystem.
2. Dashboard session cookie `auth_token` — HS256 JWT signed with `JWT_SECRET` (env or `jwt-secret` file), payload `{ authenticated: true }`, verified by `jose.jwtVerify`. Issued by `POST /api/auth/login` with the dashboard password. **Expiry is fixed at 24h (`SESSION_MAX_AGE_SEC`).**
3. `requireLogin === false` setting (unauthenticated local dashboards) — not relied upon.

Additional constraint: `POST /api/auth/login` enforces per-IP lockout (`checkLock`/`recordFail`), so 9Gauge must never spam retries.

## Decision
`gauge-core` implements a `Strategy` enum selected per host profile:

```rust
pub enum AuthStrategy {
    /// Local same-host: read machine-id + cli-secret from 9Router data dir,
    /// derive the 16-hex CLI token, send as `x-9r-cli-token` (mode: LocalCli).
    LocalCli { data_dir: PathBuf },
    /// Local or remote: dashboard password stored in OS keychain;
    /// login once, cache the JWT, attach as `Cookie: auth_token=...`,
    /// and silently re-login on the next 401 (mode: DashboardSession).
    DashboardSession { password: SecretString },
}
```

Rules:
- **Local same-host (default):** prefer `LocalCli`. Zero secrets in the keychain, no login round-trip, survives 9Router restarts.
- **Remote VPS:** `DashboardSession`. On boot (and on any 401), `POST /api/auth/login` with the keychain password, cache the JWT in memory, attach `Cookie: auth_token`. JWT is **never persisted to disk**; 24h expiry is handled by silent re-login, throttled to at most one attempt per 60s (lockout guard).
- **Bearer API keys are never used** for telemetry endpoints — verified 401.

## Consequences
- Remote auth requires the dashboard password in the OS keychain (Task 7), not an API key.
- A failed re-login must degrade the tray to Offline (rose dot), never crash the daemon or loop.
- If 9Router later grows a dedicated telemetry token (possible upstream patch), `AuthStrategy` gains a third variant without breaking the others.

## Verification Evidence
- `curl -H "Cookie: auth_token=$JWT" .../api/usage/stats?period=today` → 200 with full payload (totalRequests, byProvider, last10Minutes, recentRequests...).
- `curl -N -H "Cookie: ..." .../api/usage/stream` → `data: {...}` frames streamed.
- `curl -H "x-9r-cli-token: 48cc3ab211e44287" .../api/usage/stats` → 200.
