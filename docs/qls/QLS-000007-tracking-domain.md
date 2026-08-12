<!-- =============================================================================
File:           docs/qls/QLS-000007-tracking-domain.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Live location tracking: who can see what, and the ETA calculation it
  feeds.

Specification:
  QAS-000003, QAS-000009, QAS-000011.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000007 — Tracking Domain

**Status: Implemented.** See QAS-000003 for the underlying event
mechanism; this document covers the domain rules on top of it.

## Visibility rules

- **Admin:** `GET /v1/tracking/live` returns every courier's latest
  location within their own tenant only.
- **Customer:** `GET /v1/orders/{id}/tracking` returns the location of
  the courier assigned to *that specific order*, and only once the order
  has an `assigned_courier_id` — a customer can never query an arbitrary
  courier's location, and never a courier not assigned to their own
  order.
- **Courier:** publishes their own location
  (`POST /v1/courier/me/location`) but does not read anyone else's.

## ETA

`GET /v1/customer/orders/{id}/eta` returns `null` (not an error) when
the order has no assigned courier yet or that courier hasn't reported a
location — an expected, non-error state both native apps treat as "ETA
not available yet", not a failure. When available, the ETA uses the same
`AiDispatcher::calculate_dynamic_eta` function the dispatcher uses
internally (see QAS-000009) — the same real distance/vehicle-speed math,
no separate "customer-facing" estimate.

## Fraud flag propagation

A location flagged by the AI Fraud Guard (QAS-000009) is stored and
broadcast with `fraud_flagged=true`/`fraud_risk_score` — the admin
dashboard's live map renders a flagged courier's marker differently (red,
with a risk-score tooltip); customer-facing tracking does not surface
the fraud flag to the customer (only to operators).

## References

- [QAS-000003](../qas/QAS-000003-event-architecture.md) (the pg_notify/broadcast pipeline), [QAS-000009](../qas/QAS-000009-ai-architecture.md) (ETA/fraud
  math), [QAS-000011](../qas/QAS-000011-multi-tenant-architecture.md) (the tenant/ownership rules this enforces).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real visibility rules and ETA/fraud-flag behavior. |
