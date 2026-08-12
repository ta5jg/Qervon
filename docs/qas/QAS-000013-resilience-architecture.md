<!-- =============================================================================
File:           docs/qas/QAS-000013-resilience-architecture.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  What actually protects this system against overload/abuse and partial
  failure — rate limiting and per-request panic isolation exist; circuit
  breakers, retries, and bulkheads do not.

Specification:
  QAS-000004, QAS-000014.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000013 — Resilience Architecture

**Status: Implemented (basic) — no circuit breakers, no client-side
retry policy, single points of failure at the VPS/database level.**

## What exists

- **Rate limiting** (`tower_governor`, see QAS-000004): protects against
  abusive clients hammering credential/auth endpoints or the API in
  general; this is abuse-resistance, not failure-resistance.
- **Per-request panic isolation:** Axum/Tokio run each request on its own
  task; a panic inside one request's handler does not bring down the
  whole process or other in-flight requests (Rust's `catch_unwind`
  boundary at the task level). This is a property of the runtime, not
  code written specifically for this purpose.
- **`sqlx` connection pooling:** a bounded PostgreSQL connection pool
  (not unlimited) protects the database from unbounded connection growth
  under load; a pool-exhaustion event surfaces as a request-level error,
  not a crash.
- **Flag-and-accept, not reject, for anomalous input:** the AI Fraud
  Guard (QAS-000009) flags implausible GPS jumps rather than rejecting
  the write outright — a deliberate resilience choice: a false positive
  (a real fast vehicle, a GPS glitch) degrades gracefully to "flagged for
  review" instead of silently dropping a courier's real location.

## What does not exist

- **No circuit breakers** between `api-gateway` and PostgreSQL — a
  struggling database will cause requests to queue/timeout, not fail
  fast with a breaker.
- **No client-side retry-with-backoff policy** in the backend's own
  outbound calls (there are very few outbound calls at all today — no
  third-party payment/SMS provider is actually integrated, see
  BACKEND_BACKLOG.md, so there's little to retry against).
- **No bulkheading** — one `api-gateway` process serves every tenant;
  a resource-hungry tenant can affect others (mitigated only by the
  global rate limiter, not a per-tenant quota — see QAS-000011).
- **No multi-region/multi-AZ failover** — this runs on one VPS; see
  QAS-000015 for what disaster recovery actually means at this scale
  today.

## References

- QAS-000004 (rate limiting detail), QAS-000009 (Fraud Guard's
  flag-and-accept design), QAS-000015 (disaster recovery).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with an honest inventory of what resilience mechanisms exist and don't. |
