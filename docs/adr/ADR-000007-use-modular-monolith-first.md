<!-- =============================================================================
File:           docs/adr/ADR-000007-use-modular-monolith-first.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Architecture Decision Record: ship one deployable Rust binary
  (api-gateway) built from many internal crates/modules, rather than
  splitting into independently-deployed microservices.

Specification:
  QMI-000000, QAS-000001, QFS-000004.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# ADR-000007 — Modular Monolith First

- **Status:** Accepted — implemented.
- **Date:** 2026-08-05.
- **Deciders:** Irfan Gedik.

## Context

The source PDFs frame Qervon as eventually running "Delivery / Fleet /
Warehouse / Dispatch / Field Service / Retail / Medical Logistics"
modules, which could be read as a microservices architecture. Actually
operating N independently-deployed services requires N sets of health
checks, N deployment pipelines, and (per ADR-000006) a real event bus —
overhead this project's single-VPS, small-team reality does not justify
today.

## Decision

Structure the backend as a **modular monolith**: one Cargo workspace,
many crates, one deployable binary (`apps/api-gateway`).

- `crates/domain`, `crates/application`, `crates/infrastructure`,
  `crates/api-contracts` — the shared DDD core, compiled into every
  binary in the workspace.
- `modules/{orders,dispatch,couriers,customers,fleet,billing,
  notifications,tracking,identity}` — one crate per bounded context,
  each a thin façade over the application-layer services it needs,
  giving the *shape* of independent modules (clear crate boundaries,
  explicit dependencies) without the *cost* of independent deployment.
- `apps/api-gateway` depends on all of the above and is the only
  network-facing process; `apps/migration-runner`, `apps/bootstrap-admin`,
  and `apps/worker` are separate small binaries for operational tasks,
  sharing the same core crates.

## Consequences

- **Positive:** one binary to deploy, one health check, one log stream;
  crate boundaries still force explicit dependencies between bounded
  contexts (a compile error, not a runtime surprise, if `couriers`
  reaches into `orders`' internals); splitting a `modules/*` crate out
  into its own deployed service later is a much smaller step than
  starting from an undifferentiated monolith, because the module
  boundary already exists.
- **Negative:** cannot scale or deploy one bounded context independently
  of the others today; a bug in one module can still crash the whole
  `api-gateway` process (mitigated by Rust's panic-per-request-task
  isolation in Axum/Tokio, and to a lesser extent by comprehensive
  integration tests in `backend/apps/api-gateway/tests/`).
- **Neutral:** the AI Fraud Guard, AI Dispatcher, and ETA engine run
  in-process as plain Rust function calls (see QAS-000009), not as a
  separate "AI service" — consistent with this decision.

## Alternatives Considered

- **Microservices from day one** (per the source PDF's module list):
  rejected as premature for current team size and deployment budget (one
  VPS, no Kubernetes).
- **A single undifferentiated binary with no internal crate boundaries:**
  rejected — the DDD layering (domain/application/infrastructure) and
  per-bounded-context `modules/*` crates were kept specifically so a
  future split into real services stays plausible without a rewrite.

## References

- [QAS-000001](../qas/QAS-000001-architecture-philosophy.md) (architecture philosophy), [QFS-000004](../qfs/QFS-000004-module-system.md) (module system).
- [backend/Cargo.toml](../../backend/Cargo.toml) workspace member list.

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the actual crate structure and rationale. |
