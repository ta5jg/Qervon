<!-- =============================================================================
File:           docs/qls/QLS-000011-warehouse-domain.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  Warehouse/cross-docking-hub and cold-chain telemetry are both fully
  wired: real repository, migration, and tenant-scoped HTTP route.

Specification:
  QAS-000002, QLS-000001.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000011 — Warehouse Domain

**Status: Implemented.**

## What exists

Two real domains, each with a genuine model, passing unit tests, a
tenant-scoped repository, a governed migration, and tenant-scoped HTTP
routes (all added during the 2026-08-13 backlog closure pass):

- `backend/crates/domain/src/warehouse_hub.rs` — `WarehouseHub { id,
  tenant_id, hub_code, hub_name, location, capacity_parcels,
  active_parcels }` and `HubManifestAssignment { hub_id, courier_id,
  order_ids, ... }`, modeling a cross-docking hub where multiple parcels
  get manifested onto one courier run. Backed by
  `WarehouseHubRepository` (in-memory and Postgres) and the
  `warehouse.hubs` / `warehouse.hub_manifest_assignments` migrations;
  reachable via `POST`/`GET /v1/warehouse/hubs`,
  `POST /v1/warehouse/hubs/{id}/receive`, and
  `POST /v1/warehouse/hubs/{id}/dispatch`.
- `backend/crates/domain/src/cold_chain.rs` — temperature-controlled
  handling requirements for medical/food logistics (per the source PDFs'
  "Medical Logistics" module mention). Backed by
  `ColdChainTelemetryRepository` (in-memory and Postgres) and the
  `delivery.cold_chain_telemetry` migration; reachable via
  `POST`/`GET /v1/cold-chain/telemetry`.

Both entities carry an explicit `tenant_id`, checked the same way every
other operational endpoint in the API is (`require_operational_access`
plus a tenant-ownership check on read/write).

## What is still simplified

There is no application-layer service beyond the direct repository calls
in `http.rs`, and no mobile/web UI referencing either endpoint yet (both
are operator/integration-facing HTTP routes without a dedicated screen).
`HubManifestAssignment` has no `GET` listing route of its own — dispatched
manifests are persisted (so `warehouse.hub_manifest_assignments` is a real
audit trail) but not yet queryable through the API.

## References

- [QLS-000001](QLS-000001-logistics-domain-overview.md) (overview, listing every domain's real status),
  [BACKEND_BACKLOG.md](../../BACKEND_BACKLOG.md) (the backlog-closure record).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as an explicit Vision/Not Implemented status matching the real `// STATUS: v2 backlog` code comments. |
| 0.3.0 | 2026-08-13 | Both domains promoted out of backlog (repository, migration, tenant-scoped HTTP route); rewritten as Implemented. |
