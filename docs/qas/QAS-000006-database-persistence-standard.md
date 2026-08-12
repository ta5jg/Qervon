<!-- =============================================================================
File:           docs/qas/QAS-000006-database-persistence-standard.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  How persistence is structured: the dual memory/PostgreSQL backend, the
  migration system, and schema organization.

Specification:
  ADR-000005, ADR-000009, QAS-000002.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000006 — Database/Persistence Standard

**Status: Implemented.**

## Dual backend: memory and PostgreSQL

Every repository trait defined in `crates/domain`/`crates/application` has
two implementations in `crates/infrastructure`:

- **In-memory** (`InMemory*Repository`, guarded behind `HashMap`s and
  `RwLock`s): the default for local development and every backend
  integration test — no database process required, sub-second test runs.
- **PostgreSQL** (`Pg*Repository`, `sqlx` with compile-time checked
  queries): the production backend, selected by `QERVON_STORAGE=postgres`.

`AppState` (`apps/api-gateway/src/state.rs`) constructs the whole
dependency graph once per storage backend at startup; no handler code
branches on which backend is active.

## Migrations

`apps/migration-runner` applies SQL migration files from
`backend/migrations/<schema>/<sequence>_<name>.sql`, one directory per
logical schema (`tenancy`, `orders`, `dispatch`, `couriers`, `fleet`,
`billing`, `pricing`, `zz_cross_schema` for cross-schema foreign keys
added after the referenced schemas exist, ...), applied in a fixed phase
order (see `migration_phase` in `apps/migration-runner/src/main.rs`).
Migrations are additive and forward-only — there is no down-migration
tooling; a mistake is fixed with a new forward migration, not a rollback
script.

## Schema organization

Tables are grouped into PostgreSQL schemas matching the bounded contexts
in `modules/*` (e.g. `orders.orders`, `dispatch.assignments`,
`pricing.delivery_pricing`), rather than one flat `public` schema — this
keeps the SQL-level structure legible alongside the crate-level module
boundaries from QAS-000001.

## Multi-tenancy at the data layer

Every tenant-scoped table carries a `tenant_id` foreign key (or, for a
few entities, a join table like `vehicle_tenants`/`courier_tenants`
binding an otherwise-global entity to a tenant); every repository query
filters by `tenant_id` explicitly — there is no PostgreSQL Row-Level
Security policy doing this implicitly. See QAS-000011.

## Money and IDs

- Every monetary column is an integer minor-unit amount (`amount_minor
  BIGINT`) plus a currency code column — never `NUMERIC`/`FLOAT` for
  money (see QAS-000002).
- Every primary key is a UUIDv7 (ADR-000009), stored as PostgreSQL's
  native `uuid` type.

## What is not implemented

- No read replicas, no connection-pool-per-tenant isolation — one
  `sqlx::PgPool` shared across all tenants on one PostgreSQL instance.
- No PostGIS (see ADR-000005) — no spatial index.
- No automatic backup scheduling beyond the operator-run script in the
  backup/restore runbook (see QAS-000015).

## References

- [ADR-000005](../adr/ADR-000005-use-postgresql-postgis.md) (PostgreSQL, PostGIS not adopted), [ADR-000009](../adr/ADR-000009-use-uuid-v7.md) (UUIDv7),
  [QAS-000011](QAS-000011-multi-tenant-architecture.md) (multi-tenant architecture),
  [docs/operations/database-migration-runbook.md](../operations/database-migration-runbook.md).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real dual-backend design and migration system. |
