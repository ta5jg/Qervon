<!-- =============================================================================
File:           docs/qls/QLS-000002-order-domain.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The Order lifecycle: states, valid transitions, and who can trigger
  each one.

Specification:
  QAS-000002, QAS-000005.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000002 — Order Domain

**Status: Implemented.** Source: `backend/crates/domain/src/order.rs`.

## Lifecycle

```text
Pending → CourierAssigned → InTransit → Delivered
   ↓             ↓
Cancelled     Returned
```

| Transition | Trigger | Endpoint |
| --- | --- | --- |
| (create) → Pending | Customer creates an order | `POST /v1/customer/orders` |
| Pending → CourierAssigned | Dispatch offer accepted, or operator manual assign | `POST /v1/courier/orders/{id}/accept`, `POST /v1/orders/{id}/assign` |
| CourierAssigned → InTransit | Courier confirms pickup | `POST /v1/courier/orders/{id}/pickup` |
| InTransit → Delivered | Courier submits proof of delivery | `POST /v1/courier/orders/{id}/deliver` |
| Pending/CourierAssigned → Cancelled | Customer or operator cancels | `POST /v1/customer/orders/{id}/cancel`, `POST /v1/orders/{id}/cancel` |
| Delivered → Returned | Operator marks a return | `POST /v1/orders/{id}/return` |

An order cannot skip states (e.g. `Pending` straight to `Delivered`) —
every transition method on the domain `Order` type checks the current
state first and returns a typed error if the request doesn't apply.

## Fields

`id`, `customer_id`, `pickup`/`dropoff: Address`, `fare: Money`,
`status`, `assigned_courier_id: Option<Uuid>`, `created_at`,
`delivered_at: Option`, `returned_at: Option`, `payment_method:
Option<PaymentMethod>`, `payment_collected: bool`, `delivery_note:
Option<String>`, `contact_phone: Option<String>`.

## Fare

Never client-supplied — always computed server-side by
`PricingService::quote_fare` from the tenant's `DeliveryPricing` and the
pickup/dropoff distance, then any coupon discount applied, before the
order is persisted (see QAS-000009's AI ETA for the related distance
math, QLS-000009 for billing).

## Ownership/visibility rules

A customer sees only their own orders; a courier sees only orders
assigned to them; an admin sees every order in their own tenant, never
another tenant's (see QAS-000011).

## References

- [QAS-000002](../qas/QAS-000002-domain-model.md) (domain model), [QLS-000003](QLS-000003-dispatch-domain.md) (dispatch domain — how an order
  gets from Pending to CourierAssigned), [QLS-000013](QLS-000013-proof-of-delivery.md) (proof of delivery —
  the InTransit → Delivered evidence).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real lifecycle, transition table, and fare-computation rule. |
