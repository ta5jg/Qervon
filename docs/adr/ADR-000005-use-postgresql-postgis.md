<!-- =============================================================================
File:           docs/adr/ADR-000005-use-postgresql-postgis.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Architecture Decision Record: use PostgreSQL as the production
  datastore. PostGIS was evaluated but is not currently adopted — plain
  `double precision` lat/lng columns and Haversine distance math are used
  instead.

Specification:
  QMI-000000, QAS-000006.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# ADR-000005 — Use PostgreSQL (PostGIS Not Currently Adopted)

- **Status:** Accepted (PostgreSQL) / Not Adopted (PostGIS) — implemented.
- **Date:** 2026-08-05.
- **Deciders:** Irfan Gedik.

## Context

Qervon needs a relational store for tenant-scoped orders, couriers,
wallets, and dispatch state, with strong consistency for financial ledger
writes (wallet credits, payout approvals). It also needs to store and
query geographic coordinates (courier/order locations) and compute
distances for AI Dispatcher scoring and fare quotes.

## Decision

Use **PostgreSQL**, accessed through `sqlx` with compile-time checked
queries, migrated via a dedicated `migration-runner` binary running
per-schema migration directories in order (see QAS-000006). The backend
runs identically against an in-memory store for fast local development
and against PostgreSQL for anything durable — the same domain/application
crates drive both, only the `infrastructure` crate differs.

**PostGIS is not adopted.** Coordinates are stored as plain
`double precision latitude, longitude` columns (see e.g. the
`tenancy`/`orders`/`fleet` schemas), and distance is computed with a
Haversine formula in Rust (`qervon_domain::delivery_pricing`,
`qervon_application`'s AI Dispatcher scoring) rather than PostGIS's
`ST_Distance`/geography types. At current data volumes this is
sufficient; no spatial index (GiST) or polygon/geofence query has been
needed yet.

## Consequences

- **Positive:** one fewer PostgreSQL extension to install/manage on the
  production VPS; simpler `sqlx` query types (`f64` columns instead of
  PostGIS geometry types needing a separate Rust crate like `postgis` or
  `geo-types` bindings).
- **Negative:** no spatial indexing — a `WHERE ST_DWithin(...)` style
  "couriers within N km" query is not possible today; the current
  "closest available courier" scoring in the AI Dispatcher instead
  Haversine-scores the (currently small) set of available couriers in
  application code, which will need revisiting if the number of
  concurrently-online couriers per tenant grows into the thousands.
- **Neutral:** revisiting PostGIS remains straightforward later — it is
  additive (an extension + new geometry columns), not a data-model
  rewrite, if/when spatial-index-backed queries become necessary.

## Alternatives Considered

- **PostGIS from day one:** the originally-planned choice per the source
  PDFs; deferred because the simpler Haversine-in-Rust approach was
  sufficient for the courier counts this system runs at today, and adding
  PostGIS later is a low-cost migration if it becomes necessary.
- **MongoDB/a document store:** rejected — the domain is heavily
  relational (tenants, memberships, orders, wallets, ledger entries) and
  benefits from `sqlx`'s compile-time query checking against a real
  schema.

## References

- QAS-000006 (database/persistence standard).
- [backend/migrations/](../../backend/migrations/), `qervon_domain::delivery_pricing`.

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs, assuming PostGIS. |
| 0.2.0 | 2026-08-12 | Corrected: PostGIS is not adopted; documented the actual Haversine-in-Rust approach. |
