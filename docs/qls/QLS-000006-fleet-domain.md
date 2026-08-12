<!-- =============================================================================
File:           docs/qls/QLS-000006-fleet-domain.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The Vehicle fleet-asset entity, its lifecycle, and its tenant binding.

Specification:
  QAS-000002, QAS-000011, QLS-000004.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000006 — Fleet Domain

**Status: Implemented.** Source: `backend/crates/domain/src/vehicle.rs`,
`backend/modules/fleet`.

## Lifecycle

`VehicleStatus`: registered → `assign` → in-service → `maintenance` (and
back) → `decommission` (terminal). Endpoints:
`POST /v1/fleet/vehicles` (register/list), `GET /v1/fleet/vehicles/{id}`,
`POST /v1/fleet/vehicles/{id}/assign` (bind to a courier),
`.../maintenance`, `.../activate`, `.../decommission`.

## Fields

`id`, `plate_number`, `vehicle: VehicleType`, `status`,
`assigned_courier_id: Option<Uuid>`.

## Tenant binding (Faz-1 addition)

Vehicles originally had no explicit tenant scoping; a
`vehicle_tenants` join table and `TenantRepository::bind_vehicle`/
`find_vehicle_tenant` were added specifically to close this gap during
Faz-1's backend-hardening pass, matching the same tenant-isolation
pattern every other resource uses (see QAS-000011).

## Relationship to Courier

A `Vehicle` is optionally assigned to a `Courier` (QLS-000004) — the
courier's own `vehicle: VehicleType` field (set at registration, e.g.
"motorcycle") is independent of whether a specific fleet `Vehicle` asset
(with its own plate/maintenance record) is currently bound to them. A
courier can operate without ever having a fleet `Vehicle` bound — the
fleet module is for tracking company-owned assets, not a prerequisite for
dispatch.

## References

- QAS-000002 (domain model), QAS-000011 (the tenant-binding pattern this
  followed), QLS-000004 (courier — the relationship above).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real lifecycle and the Faz-1 tenant-binding addition. |
