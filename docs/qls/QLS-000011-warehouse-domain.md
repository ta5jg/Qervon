<!-- =============================================================================
File:           docs/qls/QLS-000011-warehouse-domain.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Warehouse/cross-docking-hub and cold-chain domain models exist as
  v2-backlog stubs — real Rust structs with tests, wired to nothing.

Specification:
  QAS-000002, QLS-000001.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000011 — Warehouse Domain

**Status: Vision / Not Implemented** (domain model + unit tests only).

## What exists

Two real Rust files, each with a genuine domain model and passing unit
tests, each carrying an explicit
`// STATUS: v2 backlog -- domain model + unit tests only; no
repository, migration, or HTTP route yet. See BACKEND_BACKLOG.md.`
comment:

- `backend/crates/domain/src/warehouse_hub.rs` — `WarehouseHub { id,
  hub_code, hub_name, location, capacity_parcels, active_parcels }` and
  `HubManifestAssignment { hub_id, courier_id, order_ids, ... }`, modeling
  a cross-docking hub where multiple parcels get manifested onto one
  courier run.
- `backend/crates/domain/src/cold_chain.rs` — temperature-controlled
  handling requirements for medical/food logistics (per the source PDFs'
  "Medical Logistics" module mention).

## What is missing to make either real

No repository trait implementation (in-memory or PostgreSQL), no
migration, no HTTP route, no application-layer service orchestrating
these types, and no mobile/web UI referencing them. None of the domain
methods on these types are reachable from outside a unit test today.

## Why this was deferred rather than half-built

Faz-1's backend-hardening scope explicitly named Warehouse as one of
several domains to defer to "v2 backlog" — writing the domain model and
tests is cheap and gives a concrete starting point; wiring a full
repository/migration/HTTP surface for a feature with no immediate
customer demand was judged not worth the time relative to the
higher-priority work (mobile apps, AI Fraud Guard, rate limiting) that
was actually done instead.

## References

- [QLS-000001](QLS-000001-logistics-domain-overview.md) (overview, listing every domain's real status),
  [BACKEND_BACKLOG.md](../../BACKEND_BACKLOG.md) (the authoritative, current v2-backlog list).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as an explicit Vision/Not Implemented status matching the real `// STATUS: v2 backlog` code comments. |
