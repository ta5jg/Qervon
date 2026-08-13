<!-- =============================================================================
File:           docs/qls/QLS-000009-billing-domain.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  Invoices, courier payouts, coupons, and tax-invoice drafts — all real
  and reachable through the HTTP API.

Specification:
  QAS-000002, QLS-000002, QLS-000004.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000009 — Billing Domain

**Status: Implemented (core + tax invoicing).**

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

## Tax invoicing (`tax_invoicing.rs`)

`TaxInvoicingEngine::generate_e_invoice` produces a tax-compliant invoice
draft, exposed via `POST /v1/tax/invoice-draft`. It is a stateless
calculator — the same category as `CurrencyExchangeEngine` — so it has no
repository or migration of its own; every field of the draft is derived
from the request (`order_id`, `customer_id`, `net_amount_minor`,
`currency`), not read back from storage. The plain `Invoice` above is
still what's actually issued/tracked per order; this endpoint produces
the tax-authority-shaped **draft** view of that amount on demand, not a
second persisted invoice record.

## References

- [QLS-000002](QLS-000002-order-domain.md) (order — fare computation this depends on), [QLS-000004](QLS-000004-courier-domain.md)
  (courier wallet — the other side of a payout), [BACKEND_BACKLOG.md](../../BACKEND_BACKLOG.md).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real invoice/payout/coupon behavior and the tax-invoicing v2-backlog status. |
| 0.3.0 | 2026-08-13 | Tax invoicing wired to `POST /v1/tax/invoice-draft`; documented as a stateless calculator, not a backlog stub. |
