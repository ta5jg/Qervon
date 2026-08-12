<!-- =============================================================================
File:           docs/adr/ADR-000009-use-uuid-v7.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Architecture Decision Record: use UUIDv7 (timestamp-ordered) for every
  generated entity identifier, via the `uuid` crate's "v7" feature.

Specification:
  QMI-000000, QAS-000006.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# ADR-000009 — Use UUIDv7 for Entity Identifiers

- **Status:** Accepted — implemented.
- **Date:** 2026-08-05.
- **Deciders:** Irfan Gedik.

## Context

Every domain entity (orders, couriers, assignments, wallet transactions,
tenants, users) needs a globally-unique, client-safe identifier. Random
UUIDv4 identifiers are safe but insert randomly into a PostgreSQL B-tree
primary-key index, causing page splits and index bloat under high insert
volume — a real cost for high-frequency tables like location updates and
wallet transactions.

## Decision

Generate every entity ID as **UUIDv7** via `uuid = { version = "1",
features = ["v7", "serde"] }` (see `backend/Cargo.toml`). UUIDv7 embeds a
millisecond Unix timestamp in its most-significant bits, so IDs generated
close together in time sort close together, which keeps B-tree inserts
mostly sequential (append-like) rather than random.

## Consequences

- **Positive:** better index locality/insert performance on high-volume
  tables (location history, wallet transactions) than UUIDv4 would give;
  IDs remain non-sequential enough that they don't leak a simple
  incrementing count the way a bare auto-increment integer would.
- **Negative:** UUIDv7's timestamp prefix means an ID technically reveals
  its approximate creation time to anyone who can decode it — judged
  acceptable here since IDs are not treated as secrets anywhere in this
  system (auth uses a separate signed-token scheme, see QAS-000004).
- **Neutral:** all identifiers across every table use the same scheme
  uniformly; there is no mix of UUIDv4 and UUIDv7 in this codebase.

## Alternatives Considered

- **UUIDv4** (fully random): the more common default; rejected due to the
  index-locality cost above at Qervon's expected insert volumes.
- **Auto-increment integers (`BIGSERIAL`):** simpler and even better for
  index locality, but leaks a sequential count (competitors could infer
  order/courier volume) and complicates multi-tenant ID issuance without
  a central sequence; rejected in favor of UUIDv7's balance of both
  properties.
- **ULID**: functionally similar to UUIDv7 (timestamp-ordered,
  128-bit); UUIDv7 was chosen because it's a standardized UUID variant
  with first-class support in the `uuid` crate already in use, avoiding
  an extra dependency.

## References

- [QAS-000006](../qas/QAS-000006-database-persistence-standard.md) (database/persistence standard).
- [backend/Cargo.toml](../../backend/Cargo.toml) (`uuid` dependency line).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the actual rationale and confirmed implementation. |
