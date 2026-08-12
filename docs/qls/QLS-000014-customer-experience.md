<!-- =============================================================================
File:           docs/qls/QLS-000014-customer-experience.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Ratings, support tickets, and notifications are real. Gamification,
  referrals, and a "favorites" concept beyond the address book are not.

Specification:
  QLS-000005, QLS-000010.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000014 — Customer Experience

**Status: Implemented (core) — gamification/referral/loyalty-redemption
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

## What is not built (`courier_leaderboard.rs` — v2 backlog)

`backend/crates/application/src/courier_leaderboard.rs` models
gamification (a courier leaderboard/ranking) with
`// STATUS: v2 backlog -- domain model + unit tests only; no
repository, migration, or HTTP route yet.` — a real model and tests, no
wiring. There is no referral-program code, no loyalty-point-redemption
flow (the points themselves exist but are never awarded — see
QLS-000005), and no "gamified" element visible to a customer or courier
in either shipped mobile app or the web pages today.

## References

- QLS-000005 (customer domain — the profile/address book underneath
  this), QLS-000010 (notifications), BACKEND_BACKLOG.md.

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten distinguishing the real ratings/support/notifications from the entirely-unbuilt gamification/referral concepts. |
