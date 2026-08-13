<!-- =============================================================================
File:           docs/qls/QLS-000001-logistics-domain-overview.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Map of the logistics domain: which of the 15 domains in this series
  are real today and which remain vision, at a glance.

Specification:
  QAS-000002.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000001 — Logistics Domain Overview

## Domain status at a glance

| Domain | Doc | Status |
| --- | --- | --- |
| Order | QLS-000002 | Implemented |
| Dispatch | QLS-000003 | Implemented |
| Courier | QLS-000004 | Implemented |
| Customer | QLS-000005 | Implemented |
| Fleet | QLS-000006 | Implemented |
| Tracking | QLS-000007 | Implemented |
| Routing (route playback) | QLS-000008 | Implemented (see `route_history.rs`) |
| Billing | QLS-000009 | Implemented (core + tax invoicing) |
| Notification | QLS-000010 | Implemented (partial — see honesty note in QLS-000010) |
| Warehouse | QLS-000011 | Implemented (see `warehouse_hub.rs`, `cold_chain.rs`) |
| Field Service | QLS-000012 | Implemented (see `field_service.rs`); no per-tenant SLA concept |
| Proof of Delivery | QLS-000013 | Implemented |
| Customer Experience | QLS-000014 | Implemented (core: ratings, support, notifications, leaderboard); loyalty-point redemption beyond accrual is Vision |
| Command Center | QLS-000015 | Implemented (the admin dashboard) |

As of the 2026-08-13 backlog closure pass (see BACKEND_BACKLOG.md), every
domain above has a real repository (or, for the two stateless calculators —
tax invoicing and currency exchange — no persistence, because there is no
entity to store), a governed migration where persistence applies, and a
tenant-scoped HTTP route. None of them carry the
`// STATUS: v2 backlog -- ...` code comment anymore; each now carries
`// STATUS: wired -- ...` instead, pointing back to BACKEND_BACKLOG.md.

## Why some domains were deferred, and why they were later promoted

Faz-1 explicitly scoped Wallet/Fleet/Order-status expansion/Fraud Guard as
the backend-hardening priority, and deliberately left
Warehouse/Cold-chain/Tax-invoicing/Gamification/Route-playback/Field-service/
Currency-exchange as "v2 backlog" at the time — real domain models with
tests (cheap to write, useful for future reference), but not wired into
repositories/migrations/HTTP routes (the expensive part), to keep the
backend-hardening phase focused. See ADR-000007's modular-monolith
discussion — a v2-backlog domain crate is exactly the kind of unit that
could become a real feature later without restructuring anything, which is
exactly what happened during the `full-vision-campaign` and its 2026-08-13
follow-up: every domain above was wired into a real repository/migration/
HTTP route with tenant-isolation tests, closing the backlog list to empty.

## References

- [QAS-000002](../qas/QAS-000002-domain-model.md) (the domain model these all extend), [BACKEND_BACKLOG.md](../../BACKEND_BACKLOG.md)
  (the backlog-closure record and any future backlog items).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as a real status map across all 15 domains in this series. |
