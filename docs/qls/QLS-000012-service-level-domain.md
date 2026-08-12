<!-- =============================================================================
File:           docs/qls/QLS-000012-service-level-domain.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  No formal SLA-tracking concept exists in this codebase. The closest
  real relative is the Field Service domain model — also a v2-backlog
  stub.

Specification:
  QAS-000002, QLS-000001.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000012 — Service Level Domain

**Status: Vision / Not Implemented.**

## No SLA-tracking concept exists

There is no code anywhere in this backend representing a Service Level
Agreement (a target delivery time, an on-time-percentage metric per
tenant, or a penalty/credit tied to missing one). If this document's
title was meant to describe that from the source PDFs, it is entirely
unbuilt — not even a stub.

## The closest real relative: Field Service

`backend/crates/application/src/field_service.rs` models scheduled
field-service jobs (per the source PDFs' "Field Service" module,
distinct from on-demand courier delivery — think a technician dispatched
for a scheduled appointment rather than an ad-hoc delivery). Like the
other domains in this state, it carries
`// STATUS: v2 backlog -- domain model + unit tests only; no
repository, migration, or HTTP route yet.` — a real model and tests, no
repository/migration/route/UI.

## What either would need to become real

- **SLA tracking:** a per-tenant target (e.g. "delivered within 45
  minutes of pickup for 95% of orders") persisted and compared against
  actual `Order.created_at`/`delivered_at` timestamps, plus a reporting
  endpoint — none of this exists as a model or a query today.
- **Field Service:** repository/migration/HTTP route wiring for the
  existing `field_service.rs` model, plus a decision on how a scheduled
  field-service job relates to the existing on-demand `Order`/dispatch
  flow (a different enough shape — scheduled vs. on-demand — that it may
  warrant its own assignment flow rather than reusing QLS-000003's).

## References

- [QLS-000001](QLS-000001-logistics-domain-overview.md) (overview), [QLS-000003](QLS-000003-dispatch-domain.md) (dispatch — the on-demand flow field
  service would need to relate to), [BACKEND_BACKLOG.md](../../BACKEND_BACKLOG.md).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as an explicit Vision/Not Implemented status, clarifying there is no SLA concept and pointing to Field Service as the nearest real stub. |
