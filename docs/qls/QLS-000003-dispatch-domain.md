<!-- =============================================================================
File:           docs/qls/QLS-000003-dispatch-domain.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  The Assignment (offer/accept/reject) flow that connects an order to a
  courier, and the AI Dispatcher scoring behind it.

Specification:
  QAS-000002, QAS-000009, QLS-000002, QLS-000004.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000003 — Dispatch Domain

**Status: Implemented.** Source:
`backend/crates/application/src/dispatch_service.rs`.

## The offer/accept/reject flow

When a customer creates an order, dispatch does not assign a courier
outright — it **offers** the job to the best-ranked available courier
(see QAS-000009 for the ranking formula):

1. `Assignment` is created with `status = Offered`, `offered_at = now`,
   a 45-second TTL. The order stays `Pending`; the courier stays
   `Available` (not yet committed).
2. The courier sees the offer via `GET /v1/courier/me/offer` (polled by
   both mobile apps every few seconds while online — see QAS-000007).
3. The courier calls `POST /v1/courier/orders/{id}/accept` (Assignment →
   `Accepted`, order → `CourierAssigned`) or `.../reject` (Assignment →
   `Rejected`, order stays `Pending`).
4. If the courier does neither within the TTL, the offer expires lazily
   — the next read of that assignment recognizes it's past `offered_at +
   45s` and treats it as `Cancelled`, without a background job actively
   sweeping expired offers.

## Automatic re-offer cascade

If a courier rejects or the offer times out, the order is automatically
re-offered to the next-best **available** courier in the same tenant —
excluding every courier already tried for this order
(`Assignment.excluded_courier_ids`, carried forward across each re-offer
via `Assignment::offer_excluding`/`excluded_including_self`,
`DispatchService::reoffer_from_candidates`). This happens lazily, at the
next trigger point (the rejecting courier's own `reject` call, or any
courier's next `GET /v1/courier/me/offer` poll discovering their offer
just expired) — there is no background sweep and no synchronous
"try every candidate in one request" loop; each response only advances
the cascade by one step.

If every available courier in the tenant has already been offered and
has rejected/expired, the order simply stays `Pending` — an operator can
still resolve it manually via `POST /v1/orders/{id}/assign` (instant
assignment, bypassing the offer flow entirely).

## Manual assignment path

`POST /v1/orders/{id}/assign` (operator-only) skips the offer negotiation
entirely and assigns a courier immediately — used for manual dispatch
when the automatic flow isn't appropriate (e.g. an operator has
out-of-band knowledge of courier availability).

## References

- [QAS-000009](../qas/QAS-000009-ai-architecture.md) (AI Dispatcher scoring that picks the offered courier),
  [QLS-000002](QLS-000002-order-domain.md) (order lifecycle this flow drives), [QLS-000004](QLS-000004-courier-domain.md) (courier
  domain — the `Available`/`Busy`/`Offline` states this flow reads).

---

# Revision History

| Version | Date | Description |
| --------- | ------ | ------------- |
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real offer/accept/reject flow and its explicit no-cascade limitation. |
| 0.3.0 | 2026-08-13 | Updated: the re-offer cascade is now implemented (`Assignment.excluded_courier_ids`, `reoffer_from_candidates`). |
