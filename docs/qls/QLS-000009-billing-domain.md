<!-- =============================================================================
File:           docs/qls/QLS-000009-billing-domain.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Invoices, courier payouts, and coupons — real; tax-specific invoicing
  is a v2-backlog stub.

Specification:
  QAS-000002, QLS-000002, QLS-000004.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000009 — Billing Domain

**Status: Implemented (core)** — tax-specific invoicing is a v2-backlog
stub, see below.

## Invoices

`Invoice { id, status: InvoiceStatus, amount: Money, ... }`
(`backend/crates/domain/src/billing.rs`), one per order, readable via
`GET /v1/customer/orders/{id}/invoice` and, in aggregate, via
`GET /v1/finance/invoices` (operator-facing). `GET /v1/finance/summary`
aggregates invoiced totals and approved-payout totals **by currency**
(a tenant operating in multiple currencies gets a correct multi-currency
summary, not a naive sum across currencies).

## Courier payouts

`CourierPayout { status: PayoutStatus, ... }` records that a payout was
approved for a courier — this is a bookkeeping record, not a real
bank-transfer integration; no money actually moves through this system
(see QLS-000004's wallet section and BACKEND_BACKLOG.md).

## Coupons

`Coupon` (`backend/crates/domain/src/coupon.rs`) — real, tenant-scoped
discount codes, created via `POST /v1/coupons` (operator), applied via
`coupon_code` on `POST /v1/customer/orders`.
`PromoCouponEngine::apply_to_fare` redeems the coupon and computes the
discounted fare **at order-creation time** — there is no separate
no-side-effect "preview a coupon's discount before committing" endpoint;
previewing requires actually creating the order (see
BACKEND_BACKLOG.md's Faz-2.3 boundary note).

## Tax invoicing (`tax_invoicing.rs`) — v2 backlog

A real domain model for tax-compliant invoice formatting exists but
`// STATUS: v2 backlog -- domain model + unit tests only; no
repository, migration, or HTTP route yet.` — the plain `Invoice` above is
what's actually issued today; a tax-authority-compliant invoice format
(sequential numbering rules, required fields for a specific
jurisdiction, etc.) is not implemented.

## References

- QLS-000002 (order — fare computation this depends on), QLS-000004
  (courier wallet — the other side of a payout), BACKEND_BACKLOG.md.

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real invoice/payout/coupon behavior and the tax-invoicing v2-backlog status. |
