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
| Routing (route playback) | QLS-000008 | **Vision** (domain model + tests only, see `route_history.rs`) |
| Billing | QLS-000009 | Implemented (core); tax invoicing is a v2-backlog subset |
| Notification | QLS-000010 | Implemented (partial — see honesty note in QLS-000010) |
| Warehouse | QLS-000011 | **Vision** (domain model + tests only, see `warehouse_hub.rs`, `cold_chain.rs`) |
| Field Service | QLS-000012 | **Vision** (domain model + tests only, see `field_service.rs`) |
| Proof of Delivery | QLS-000013 | Implemented |
| Customer Experience | QLS-000014 | Implemented (core: ratings, support, notifications); gamification/loyalty beyond points is Vision |
| Command Center | QLS-000015 | Implemented (the admin dashboard) |

A domain marked "domain model + tests only" in this codebase has a real
Rust struct with real unit tests, but no repository implementation, no
database migration, and no HTTP route — it cannot be exercised through
the API today. Each such file carries an explicit
`// STATUS: v2 backlog -- ...` comment; see BACKEND_BACKLOG.md for the
full, current list and the reasoning behind deferring each one.

## Why some domains were deferred rather than half-built

Faz-1 explicitly scoped Wallet/Fleet/Order-status expansion/Fraud Guard as
the backend-hardening priority, and deliberately left
Warehouse/Cold-chain/Tax-invoicing/Gamification/Route-playback/Field-service/
Currency-exchange as "v2 backlog" — real domain models with tests
(cheap to write, useful for future reference), but not wired into
repositories/migrations/HTTP routes (the expensive part), to keep the
backend-hardening phase focused. See ADR-000007's modular-monolith
discussion — a v2-backlog domain crate is exactly the kind of unit that
could become a real feature later without restructuring anything.

## References

- QAS-000002 (the domain model these all extend), BACKEND_BACKLOG.md
  (the current, authoritative v2-backlog list).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as a real status map across all 15 domains in this series. |
