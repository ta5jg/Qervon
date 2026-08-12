<!-- =============================================================================
File:           docs/qfs/QFS-000003-runtime-lifecycle.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The real startup/shutdown sequence of the api-gateway process.

Specification:
  QAS-000014, QFS-000006.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QFS-000003 — Runtime Lifecycle

**Status: Implemented.** Source: `backend/apps/api-gateway/src/main.rs`.

## Startup sequence

1. Initialize `tracing_subscriber` (env-filter from `RUST_LOG`, default
   `qervon_api_gateway=info,tower_http=info`).
2. Read `QERVON_LISTEN` (default `0.0.0.0:8080`).
3. `AppState::from_env()` — reads every `QERVON_*` environment variable
   (see QFS-000006), constructs the storage backend (in-memory or a real
   `sqlx::PgPool`, see QAS-000006), and wires every application service.
   This is a fallible step — a missing required variable
   (`QERVON_TOKEN_SIGNING_SECRET`/`QERVON_API_ACCESS_TOKEN`) causes the
   process to exit immediately with an error rather than start in a
   half-configured state.
4. Build the Axum router (`router(state)` — CORS and rate-limit layers
   applied here, see QAS-000004).
5. Bind the TCP listener, then `axum::serve(...)` — the process now
   accepts connections.

There is no separate "ready" phase distinct from this — `GET /ready`
(QAS-000012) reports readiness based on whether auth is configured, not
based on any additional startup step after step 5.

## Shutdown

**No graceful shutdown handler is registered** (`axum::serve` is called
without `.with_graceful_shutdown(...)`). A `systemctl stop`/SIGTERM ends
the process immediately — in-flight requests are dropped, not drained.
For a stateless-per-request HTTP API this is a minor issue (a client
sees a connection error and can retry), but it is a real gap worth
closing before this system needs zero-downtime deploys: adding a
`tokio::signal::ctrl_c()`/SIGTERM listener that stops accepting new
connections and waits briefly for in-flight ones to finish is a small,
well-understood change.

## Other binaries in this workspace

- `apps/migration-runner`: runs once, applies pending migrations, exits.
  No long-running server.
- `apps/worker`: same startup shape as `api-gateway` (env-based config,
  `tracing` init) but then enters a polling loop instead of serving HTTP
  — see QFS-000011.
- `apps/bootstrap-admin`: runs once, creates the first tenant/admin,
  exits — see QAS-000014's deployment procedure.

## References

- [QFS-000006](QFS-000006-configuration-system.md) (configuration — what `from_env()` reads), [QAS-000014](../qas/QAS-000014-deployment-architecture.md)
  (deployment — how this process is supervised by systemd), [QFS-000011](QFS-000011-scheduler.md)
  (the worker's lifecycle).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real startup sequence and the honest lack of graceful shutdown. |
