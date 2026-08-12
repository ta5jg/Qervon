<!-- =============================================================================
File:           docs/qes/QES-000013-performance-engineering.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Real performance-relevant decisions made so far, and the honest
  absence of load-testing/profiling infrastructure.

Specification:
  ADR-000005, ADR-000009, QAS-000013.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000013 — Performance Engineering

**Status: Implemented (a few concrete decisions) — no load testing, no
profiling infrastructure exists yet.**

## Decisions made specifically for performance

- **UUIDv7 over UUIDv4** (ADR-000009) — better B-tree index locality on
  high-insert tables (location history, wallet transactions).
- **In-process broadcast channel for live location** (QAS-000003) —
  avoids a per-viewer database read on every location update; only the
  write path touches PostgreSQL, every WebSocket/poll reader is served
  from memory.
- **Rust + `sqlx` compile-time checked queries** (ADR-000001) — no ORM
  overhead, predictable query plans the developer wrote by hand.
- **Bounded connection pool** (QAS-000013) — protects PostgreSQL from
  unbounded connection growth under load.
- **No pagination yet** (QAS-000005) is a known *future* performance
  liability, not a current one — acceptable at today's data volumes,
  explicitly flagged as needing revisiting before it becomes a problem.

## What is not implemented

- No load-testing suite (no k6/Locust/Gatling scripts in this
  repository) — performance claims here are based on architectural
  reasoning, not measured benchmarks.
- No APM/profiling wired in (see QAS-000012) — a slow endpoint would be
  diagnosed today via `tracing` log timestamps and manual `EXPLAIN
  ANALYZE`, not a flame graph.
- No caching layer (no Redis, no in-process cache beyond the location
  broadcast mechanism above) for read-heavy endpoints.
- No CDN for the shipped web pages' static assets — they're served
  directly by `api-gateway` via `include_str!`, and third-party assets
  (fonts, Leaflet, lucide) come from their own public CDNs, not a
  Qervon-controlled one.

## References

- ADR-000009 (UUIDv7), QAS-000003 (event architecture), QAS-000013
  (resilience — the closely related "what happens under load" document).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real performance-motivated decisions and an honest list of missing tooling. |
