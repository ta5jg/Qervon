<!-- =============================================================================
File:           docs/qls/QLS-000012-service-level-domain.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  No formal SLA-tracking concept exists in this codebase. The closest
  real relative is the Field Service domain, which is now fully wired
  (repository, migration, tenant-scoped HTTP route).

Specification:
  QAS-000002, QLS-000001.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000012 — Service Level Domain

**Status: Vision / Partially Implemented (Field Service only).**

## No SLA-tracking concept exists

There is no code anywhere in this backend representing a Service Level
Agreement (a target delivery time, an on-time-percentage metric per
tenant, or a penalty/credit tied to missing one). If this document's
title was meant to describe that from the source PDFs, it is entirely
unbuilt — not even a stub. The closest thing to an SLA-style measure is
the courier leaderboard's "on-time" definition (delivered within 60
minutes of order creation, see `GET /v1/couriers/leaderboard` and
QLS-000004), which is a per-courier scoring input, not a per-tenant SLA.

## The closest real relative: Field Service

`backend/crates/domain/src/field_service.rs` models scheduled
field-service jobs (per the source PDFs' "Field Service" module,
distinct from on-demand courier delivery — think a technician dispatched
for a scheduled appointment rather than an ad-hoc delivery). As of the
2026-08-13 backlog closure it is fully wired: a `tenant_id`-scoped
`FieldServiceAppointmentRepository` (in-memory and Postgres), the
`service.field_service_appointments` migration, and tenant-scoped HTTP
routes (`POST`/`GET /v1/field-service/appointments`). It moved from
`qervon-application` to `qervon-domain` as part of that pass so its
repository trait could live alongside every other one in
`repository.rs`.

## What SLA tracking would still need to become real

- A per-tenant target (e.g. "delivered within 45 minutes of pickup for
  95% of orders") persisted and compared against actual
  `Order.created_at`/`delivered_at` timestamps, plus a reporting
  endpoint — none of this exists as a model or a query today.
- A decision on how a scheduled field-service job relates to the
  existing on-demand `Order`/dispatch flow (a different enough shape —
  scheduled vs. on-demand — that it may warrant its own assignment flow
  rather than reusing QLS-000003's) is also still open; Field Service
  appointments today have an optional `technician_id` but no dispatch
  offer/accept flow of their own.

## References

- [QLS-000001](QLS-000001-logistics-domain-overview.md) (overview), [QLS-000003](QLS-000003-dispatch-domain.md) (dispatch — the on-demand flow field
  service would need to relate to), [QLS-000004](QLS-000004-courier-domain.md) (courier leaderboard's on-time measure), [BACKEND_BACKLOG.md](../../BACKEND_BACKLOG.md).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as an explicit Vision/Not Implemented status, clarifying there is no SLA concept and pointing to Field Service as the nearest real stub. |
| 0.3.0 | 2026-08-13 | Field Service promoted out of backlog (repository, migration, HTTP route); updated status and moved-file reference. |
