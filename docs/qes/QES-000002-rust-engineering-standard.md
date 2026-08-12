<!-- =============================================================================
File:           docs/qes/QES-000002-rust-engineering-standard.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Rust conventions actually enforced/observed across the backend
  workspace: crate layout, error handling, and testing patterns.

Specification:
  ADR-000001, QAS-000001, QES-000006.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000002 — Rust Engineering Standard

**Enforced by CI** (`.github/workflows/backend.yml`): `cargo fmt --check`,
`cargo clippy --workspace --all-targets -- -D warnings` (warnings are
build failures, not suggestions), `cargo test --workspace --all-targets`.

## Crate layout

Follow the layering in QAS-000001: a change to a business rule goes in
`crates/domain`; orchestration of multiple domain objects/repositories
goes in `crates/application`; a new persistence implementation goes in
`crates/infrastructure`; wire DTOs go in `crates/api-contracts`. A
`modules/*` crate should stay a thin façade — if it accumulates real
business logic, that logic belongs in `application` instead.

## Error handling

- Domain/application errors are typed enums (e.g. `OrderError`,
  `DispatchError`), not `anyhow::Error` strings — callers should be able
  to `match` on a specific failure mode.
- The HTTP boundary (`api-gateway`) converts these into `ApiError`
  (`{status, title, detail}` — see QAS-000005) exactly once, at the
  handler level; lower layers never construct an `ApiError` or know about
  HTTP status codes.
- `.unwrap()`/`.expect()` are acceptable only for genuinely-impossible
  states (e.g. a `Duration::from_secs(45)` construction) or in tests —
  never on a value derived from request input or a database read.

## Database access

- Use `sqlx::query!`/`query_as!` (compile-time checked against a real
  schema) wherever the query is static; drop to `sqlx::query` only when
  building a genuinely dynamic query.
- Every repository trait gets both an in-memory and a PostgreSQL
  implementation (see QAS-000006) — a new repository method is not
  "done" until both exist and both pass the same test.

## Testing

- Unit tests live beside the code they test (`#[cfg(test)] mod tests`),
  exercising domain/application logic against the in-memory
  infrastructure (fast, no external process).
- Integration tests (`backend/apps/api-gateway/tests/api_flow.rs`) drive
  the real HTTP router end-to-end for a full user flow (e.g. "customer
  creates an order, courier accepts, delivers, wallet is credited") —
  this is where cross-tenant isolation and auth are actually verified,
  not just unit-tested in isolation.
- `make test-postgres` runs the same integration suite against a real
  PostgreSQL instance — both backends must pass the same behavioral
  tests (see QAS-000006).

## Documentation comments

Public functions in `domain`/`application` get a `///` doc comment
explaining the business rule, not just the Rust signature — e.g.
`AiDispatcher::calculate_score`'s comment names the actual scoring
formula, not just "computes a score".

## References

- [ADR-000001](../adr/ADR-000001-use-rust-for-backend.md) (why Rust), [QAS-000001](../qas/QAS-000001-architecture-philosophy.md) (the layering this enforces),
  [QES-000006](QES-000006-testing-standard.md) (testing standard, the fuller version of the section above).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real, CI-enforced Rust conventions. |
