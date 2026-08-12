<!-- =============================================================================
File:           docs/qls/QLS-000010-notification-domain.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Notification records and device-push registration are real; actually
  sending an SMS/push/email is not.

Specification:
  QAS-000002, QAS-000005.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000010 — Notification Domain

**Status: Implemented (record-keeping) — no real outbound delivery to
any channel.**

## What is real

- `Notification { recipient_id, channel: NotificationChannel, title,
  body, status: NotificationDeliveryStatus, ... }` — a persisted record,
  readable via `GET /v1/customer/notifications`.
- `DevicePushToken` registration —
  `POST /v1/push/devices` (both mobile platforms register a real device
  token when one is available; see QAS-000007), `DELETE
  /v1/push/devices/{id}`, `GET /v1/push/config` (VAPID key for the
  web-push flow the shipped web pages use, see `qervon-client.js`
  callers in `mobile-customer.html`/`mobile-courier.html`).
- Browser web-push subscriptions (`POST /v1/push/subscriptions`) — the
  one channel that is genuinely end-to-end real, because it uses the
  standard Web Push API with no third-party credential requirement
  beyond the VAPID keypair this backend generates itself.

## What is not real

- **SMS:** OTP codes are generated and verified, but never sent via SMS
  — see BACKEND_BACKLOG.md. `NotificationChannel::Sms` exists as an enum
  variant with no sending implementation behind it.
- **Native push (APNs/FCM):** device tokens are collected (real), but
  nothing sends an actual push notification to them — no Apple/Google
  push credential exists in this environment (see QAS-000007's mobile
  honesty notes).
- **WhatsApp/Email:** enum variants exist (`NotificationChannel::Whatsapp`,
  `::Email`); no provider integration for either.

This mirrors the existing `backend/apps/worker`'s "register vs. send" gap
intentionally — collecting a subscription/token is real and useful on
its own (it's the hard part to get right, security-wise), while the
actual send is a thin, swappable integration once a real credential
exists.

## References

- QAS-000005 (API conventions), QAS-000007 (mobile push-token capture),
  BACKEND_BACKLOG.md (the SMS/push-sending gap).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten distinguishing the real record-keeping/registration from the entirely-absent SMS/native-push sending. |
