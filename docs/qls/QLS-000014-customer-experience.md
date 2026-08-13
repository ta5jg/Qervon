<!-- =============================================================================
File:           docs/qls/QLS-000014-customer-experience.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  Ratings, support tickets, notifications, and the courier gamification
  leaderboard are real. Referrals and loyalty-point redemption are not.

Specification:
  QLS-000005, QLS-000010.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000014 — Customer Experience

**Status: Implemented (core + gamification) — referral/loyalty-redemption
are not built.**

## What is real

- **Ratings:** `POST /v1/customer/orders/{id}/rating` (1–5 stars +
  comment) after delivery — see QLS-000004.
- **Support tickets:** `POST`/`GET /v1/customer/support-tickets`,
  optionally linked to a specific order (`order_id: Option<Uuid>`),
  with a `status: TicketStatus` (`Open/InProgress/Resolved/Closed`).
- **Notifications:** `GET /v1/customer/notifications` (see QLS-000010
  for what's real vs. not about actually *sending* one).
- **Address book / "favorites":** the only "favorite" concept that
  exists is the saved address book (QLS-000005) — there is no separate
  favorite-courier or favorite-order feature.
- **Courier gamification leaderboard:** `GET /v1/couriers/leaderboard`
  (see below) — the one gamification element that is real.

## Courier leaderboard (`courier_leaderboard.rs`)

`backend/crates/application/src/courier_leaderboard.rs` models a
composite performance score and rank
(`completed_deliveries * 10 + on_time_rate% * 5 + average_rating * 50`).
It is exposed as a tenant-scoped read model at
`GET /v1/couriers/leaderboard`: every input is computed live from the
existing `OrderRepository` and `CustomerRatingRepository` rather than
duplicated into a new table, and "on-time" is defined as delivered within
60 minutes of order creation. It has no repository or migration of its
own by design — see BACKEND_BACKLOG.md.

## What is still not built

There is no referral-program code, and no loyalty-point-redemption flow
(the points themselves exist but are never awarded — see QLS-000005). The
leaderboard above is the only "gamified" element visible through the API
today; neither shipped mobile app nor the web pages surface it in a
dedicated screen yet.

## References

- [QLS-000005](QLS-000005-customer-domain.md) (customer domain — the profile/address book underneath
  this), [QLS-000010](QLS-000010-notification-domain.md) (notifications), [BACKEND_BACKLOG.md](../../BACKEND_BACKLOG.md).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten distinguishing the real ratings/support/notifications from the entirely-unbuilt gamification/referral concepts. |
| 0.3.0 | 2026-08-13 | Courier leaderboard wired to `GET /v1/couriers/leaderboard` as a live read model; referral/loyalty-redemption remain unbuilt. |
