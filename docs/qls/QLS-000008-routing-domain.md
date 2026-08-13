<!-- =============================================================================
File:           docs/qls/QLS-000008-routing-domain.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  Honest status of route optimization and route-history/playback —
  playback is now fully wired end-to-end; multi-stop route optimization
  still does not exist at all.

Specification:
  QAS-000009, QLS-000001.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000008 — Routing Domain

**Status: Partially Implemented** (route-history/playback is real and
wired; route optimization has nothing).

## Route history / playback (`route_history.rs`)

A real Rust domain model exists
(`backend/crates/domain/src/route_history.rs`) representing a recorded
sequence of a courier's positions for later playback (e.g. an operator
reviewing "where did this courier actually go during this delivery"). As
of the 2026-08-13 backlog closure it is fully wired: a `tenant_id`-scoped
`RouteBreadcrumbRepository` (in-memory and Postgres), the
`tracking.route_breadcrumbs` migration, and tenant-scoped HTTP routes
(`POST /v1/route-history/{courier_id}/breadcrumbs` and
`GET /v1/route-history/{courier_id}?date=YYYY-MM-DD`). Recording a
breadcrumb requires the courier to already be bound to the caller's
tenant (`TenantRepository::find_courier_tenant`), the same ownership check
every other courier-scoped write in the API uses.

## Multi-stop route optimization ("AI Route")

Mentioned in the source PDFs as "AI Route: bir kuryeye 5–10 teslimatı en
verimli sırayla planlar" (plan 5–10 deliveries for one courier in the
most efficient order). **No code for this exists at all** — not even a
stub domain model. The AI Dispatcher (QAS-000009) assigns one order to
one courier at a time; there is no concept of batching multiple orders
onto a single courier run or sequencing stops.

## What building multi-stop route optimization for real would need

- A `Route { stops: Vec<RouteStop> }` domain model wired to
  repositories/migrations/HTTP routes the same way `route_history.rs` now
  is, and a real route-optimization algorithm (even a simple
  nearest-neighbor heuristic would be a legitimate first version — a
  full TSP solver is not required to be useful).
- A decision on how this interacts with the existing single-order offer
  flow (QLS-000003) — does a courier accept a batch of orders as one
  unit, or does route optimization happen after several individual
  orders are already assigned to the same courier?

## References

- [QAS-000009](../qas/QAS-000009-ai-architecture.md) (what AI Dispatcher actually does today), [QLS-000001](QLS-000001-logistics-domain-overview.md)
  (overview of which domains are real vs. vision), [BACKEND_BACKLOG.md](../../BACKEND_BACKLOG.md).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as an explicit Vision/Not Implemented status distinguishing the route-history stub from the entirely-absent route-optimization feature. |
| 0.3.0 | 2026-08-13 | Route-history/playback promoted out of backlog (repository, migration, tenant-scoped HTTP route); route optimization remains unbuilt. |
