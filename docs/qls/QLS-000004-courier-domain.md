<!-- =============================================================================
File:           docs/qls/QLS-000004-courier-domain.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The Courier entity: status states, wallet crediting, and ratings.

Specification:
  QAS-000002, QLS-000003, QLS-000006.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000004 — Courier Domain

**Status: Implemented.**

## Status states

`Available ↔ Busy ↔ Offline`, toggled by `POST /v1/courier/me/status`
(online/offline) and implicitly by dispatch (an accepted job makes a
courier effectively unavailable for a new offer while it's in progress —
enforced by the AI Dispatcher only ranking couriers with a known,
plausible current location and, in practice, one active assignment at a
time).

## Location

`current_location: Option<Location>`, updated by
`POST /v1/courier/me/location`, passed through the AI Fraud Guard's
speed-anomaly check on every write (see QAS-000009) before being stored
and broadcast (see QAS-000003).

## Wallet (`CourierWallet`)

One wallet per courier, credited **automatically** on delivery completion
— no manual "pay the courier" step. `WalletTransactionType`:
`DeliveryEarning`, `PerformanceBonus`, `Tip`, `PenaltyDeduction`,
`PayoutWithdrawal`. `GET /v1/courier/me/wallet` returns the balance plus
full transaction history; `GET /v1/couriers/{id}/wallet` is the
admin-facing equivalent. There is no real payout/bank-transfer
integration — a `PayoutWithdrawal` transaction records that a payout was
approved, it does not move real money (see BACKEND_BACKLOG.md).

## Ratings

`CustomerRating` (1–5 stars + optional comment), created via
`POST /v1/customer/orders/{id}/rating` after delivery, readable by the
courier (`GET /v1/courier/me/ratings`, self-access only) and by an admin
(`GET /v1/couriers/{id}/ratings`). No aggregate "average rating" field is
persisted — both native apps compute the average client-side from the
returned list (see QAS-000007).

## Provisioning

A courier account is always created by a tenant admin
(`POST /v1/couriers/provision`), never through public self-registration
— see QAS-000011's provisioning rules.

## Vehicle association

A courier's `vehicle: VehicleType` (Bicycle/Motorcycle/Car) is set at
registration; a separate, optional `Vehicle` fleet asset (with its own
plate/insurance/maintenance record) can be bound to a courier — see
QLS-000006.

## References

- [QAS-000002](../qas/QAS-000002-domain-model.md) (domain model), [QLS-000003](QLS-000003-dispatch-domain.md) (dispatch — how a courier
  receives work), [QLS-000006](QLS-000006-fleet-domain.md) (fleet — the vehicle asset relationship).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real status/wallet/ratings behavior. |
