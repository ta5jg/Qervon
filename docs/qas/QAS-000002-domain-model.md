<!-- =============================================================================
File:           docs/qas/QAS-000002-domain-model.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The core domain entities and value objects in qervon_domain, and how
  they relate.

Specification:
  QAS-000001, QLS-000001.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000002 — Domain Model

**Status: Implemented.** Source of truth: `backend/crates/domain/src/`.

## Core entities

| Entity | Key fields | Lifecycle |
| --- | --- | --- |
| `Tenant` | id, slug, name | created at provisioning; every other entity below is tenant-scoped |
| `Order` | pickup/dropoff `Address`, `fare: Money`, `status`, `payment_method`, `delivery_note`, `contact_phone` | `Pending → CourierAssigned → InTransit → Delivered`, or `→ Cancelled`/`→ Returned` at defined points (see QLS-000002) |
| `Courier` | name, `vehicle: VehicleType`, `status: CourierStatus`, `current_location` | `Available ↔ Busy ↔ Offline` |
| `Assignment` | order+courier pair, `status` (`Offered/Accepted/Rejected/Cancelled`), `offered_at`/`responded_at` | the offer/accept/reject negotiation between dispatch and a courier (see QLS-000003) |
| `CustomerProfile` | `addresses: Vec<SavedAddress>`, `loyalty_points` | one per customer user |
| `CourierWallet` | `balance_minor`, `total_earned_minor`, `transactions: Vec<WalletTransaction>` | credited automatically on delivery completion |
| `DeliveryPricing` | `base_fare`, `per_km_rate`, `min_fare` | one per tenant, defaults applied if unconfigured |
| `Vehicle` | plate, type, `status: VehicleStatus` | fleet asset, optionally assigned to a courier |
| `Coupon` | code, discount rule | redeemed against a fare at order-creation time |
| `SupportTicket`, `CustomerRating`, `Notification`, `DevicePushToken` | — | customer-feedback and messaging records |

## Value objects

- `Money { amount_minor, currency }` — every monetary amount in the system
  is an integer minor-unit count (kuruş) plus an ISO currency string;
  there is no floating-point money anywhere in the domain.
- `Location { latitude, longitude }` and `Address { location, label }` —
  `label` is free-text and always HTML-escaped by any HTML-rendering
  client before display (see QAS-000008's security notes — this is where
  the one real XSS gap found in this codebase originated).
- Every entity ID is a `Uuid` generated as UUIDv7 (ADR-000009).

## What this model deliberately does not include

- No generic "custom fields"/schema-less extension mechanism — adding a
  new attribute to `Order` means a real Rust struct field and a real
  migration, not a JSON blob column. This is a deliberate rejection of
  the schema-less flexibility a "rule engine"/"plugin system" style
  design might offer (see QFS-000001) in favor of compile-time-checked
  correctness.
- No `Warehouse`, `RouteStep`, or `FieldServiceJob` entities — these
  domains from the source vision PDFs are not built; see QLS-000011 and
  QLS-000012.

## References

- [QAS-000001](QAS-000001-architecture-philosophy.md) (layering these entities live inside), [QAS-000006](QAS-000006-database-persistence-standard.md)
  (how they're persisted), [QLS-000001](../qls/QLS-000001-logistics-domain-overview.md) through [QLS-000015](../qls/QLS-000015-command-center.md) (per-domain
  detail).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the actual entity/value-object list from qervon_domain. |
