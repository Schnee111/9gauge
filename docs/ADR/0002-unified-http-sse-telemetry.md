# ADR 0002: Unified HTTP SSE & REST Ingestion over Direct Local SQLite Reads

## Status
Accepted

## Context
9Gauge must ingest real-time request pulses and historical token/cost aggregations from 9Router. Because 9Router stores all telemetry in a local SQLite file (`~/.9router/db/data.sqlite`), two data access patterns were analyzed:
1. **Direct SQLite Reads (`mode=ro`)** via Rust `rusqlite` with `inotify`/polling.
2. **Unified Event-Driven HTTP (SSE Stream + Cached REST APIs)** via 9Router's internal HTTP endpoints.

## Decision
We choose **Unified Event-Driven HTTP (SSE Stream + Cached REST)**.

## Rationale & Trade-Offs

### Risks of Direct SQLite Reads
- **WAL Checkpoint Starvation:** High-frequency read queries from 9Gauge on the active WAL file can block 9Router's checkpointing thread during heavy AI agent request bursts, leading to WAL file bloat.
- **Environment Fragmentation:** Direct disk reads fail completely when monitoring a remote 9Router instance on a VPS (`https://9router.aeter.my.id`) or when 9Router runs inside an isolated Docker container without shared volume mounts.
- **Battery Impact:** Polling the SQLite file or watching directory inodes triggers frequent CPU wakeups, violating macOS App Nap and battery efficiency standards.

### Benefits of Unified HTTP (SSE + REST)
- **Zero Disk I/O:** Leverages kernel socket buffers over localhost loopback (`127.0.0.1:20128`) without hitting storage.
- **Identical Pipeline for Local & Remote:** The exact same client code connects to `localhost:20128` (via `x-9r-cli-token`) or `vps.domain.com` (via Bearer token).
- **Minimal Wakeups:** SSE keepalive generates only ~2.4 wakeups/minute during idle periods, delivering maximum energy preservation on laptops.
