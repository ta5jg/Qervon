<!-- =============================================================================
File:           docs/adr/ADR-000001-use-rust-for-backend.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Architecture Decision Record: use Rust for the Qervon backend.

Specification:
  QMI-000000, QAS-000001, QES-000002.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# ADR-000001 — Use Rust for the Backend

- **Status:** Accepted — implemented.
- **Date:** 2026-08-05.
- **Deciders:** Irfan Gedik.

## Context

Qervon's backend is the load-bearing part of the product: it holds
multi-tenant order/dispatch state, streams live courier locations to
potentially many concurrent viewers, and must run affordably on a single
VPS rather than a large managed-services bill. The language choice needed
to give predictable latency under concurrent load, a memory-safety
guarantee strong enough to trust with financial data (wallets, invoices),
and a realistic path to a single, small, statically-linked deployment
artifact.

## Decision

Build the entire backend in Rust, structured as a Cargo workspace
(`backend/`) with:

- `crates/domain`, `crates/application`, `crates/infrastructure`,
  `crates/api-contracts` — the Domain-Driven Design core (see
  QAS-000001, QAS-000002).
- `modules/*` — vertical slices (`orders`, `dispatch`, `couriers`,
  `customers`, `fleet`, `billing`, `notifications`, `tracking`,
  `identity`) that depend on the core crates (see QFS-000004).
- `apps/api-gateway` — the Axum-based HTTP server, the only network-facing
  binary.
- `apps/migration-runner`, `apps/bootstrap-admin`, `apps/worker` —
  operational binaries sharing the same core crates.

Key crates actually in the dependency tree today: `axum` 0.8 (HTTP),
`tokio` (async runtime), `sqlx` (PostgreSQL, compile-time checked queries),
`tower`/`tower-http` (CORS, tracing, rate limiting via `tower_governor`),
`serde`/`serde_json` (wire format), `utoipa`/`utoipa-swagger-ui` (OpenAPI
generation and the `/swagger-ui` route), `argon2` (password hashing),
`hmac`/`sha2` (the custom `qv1.<payload>.<signature>` access token —
see QAS-000004), `uuid` with the `v7` feature (see ADR-000009), `chrono`
(timestamps), `tracing`/`tracing-subscriber` (structured logs).

## Consequences

- **Positive:** a single `cargo build --release` produces one dependency-free
  binary per app; the compiler rejects most concurrency and null-pointer
  bugs at compile time; `sqlx::query!`/`query_as!` catch SQL/schema
  mismatches at compile time against a real database, not at runtime; the
  same core crates run against either an in-memory store (fast local dev,
  see QAS-000006) or PostgreSQL (production) with no code duplication.
- **Negative:** slower iteration speed than a dynamically-typed stack for
  small UI-adjacent changes (mitigated by pushing all UI logic into the
  vanilla-HTML/JS web layer and native mobile apps, never into the Rust
  binary); Rust's learning curve is real and this is a small team.
- **Neutral:** no async runtime other than Tokio was seriously evaluated;
  no non-Rust backend framework was evaluated once the language was fixed.

## Alternatives Considered

- **Node.js/TypeScript**: faster initial velocity, but weaker guarantees
  around the concurrent GPS-stream and wallet-ledger code paths, and a much
  heavier runtime footprint on a budget VPS.
- **Go**: a reasonable alternative with similar deployment properties;
  Rust was preferred for its stronger type system (no `nil` panics) and
  `sqlx`'s compile-time query checking, which Go's ecosystem does not
  match as directly.

## References

- [QAS-000001](../qas/QAS-000001-architecture-philosophy.md) (architecture philosophy), [QAS-000002](../qas/QAS-000002-domain-model.md) (domain model),
  [QES-000002](../qes/QES-000002-rust-engineering-standard.md) (Rust engineering standard).
- [backend/README.md](../../backend/README.md), [root README.md](../../README.md).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the actual decision, rationale, and implemented crate structure. |
