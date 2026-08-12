<!-- =============================================================================
File:           docs/qfs/QFS-000014-integration-runtime.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Real outbound webhooks (customer-configurable, HTTPS-only, delivered
  via the worker's outbox pattern) — no inbound third-party API-key
  integration surface exists yet.

Specification:
  QFS-000011, QAS-000005.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QFS-000014 — Integration Runtime

**Status: Implemented (outbound webhooks) — no inbound third-party
integration surface.**

## Outbound webhooks (real)

`WebhookSubscription { tenant_id, endpoint_url, event_types, secret_hash,
enabled }` (`backend/crates/domain/src/webhook.rs`), managed via
`POST`/`GET /v1/customer/webhooks`, `DELETE
/v1/customer/webhooks/{id}`. Real validation at creation: the endpoint
**must** be HTTPS (plain HTTP is rejected outright), at least one event
type must be specified, and the stored secret is a hash, never the raw
secret (`QERVON_WEBHOOK_ENCRYPTION_KEY` encrypts it at rest, see
QFS-000006).

Delivery goes through the same outbox/polling worker described in
QFS-000011 — an event writes a delivery-attempt row inside the
triggering transaction, and the worker drains it, rather than an
in-request HTTP call to the customer's endpoint (which would make order
creation slower and less reliable, coupling it to a third party's
uptime).

## What is not implemented

- **No inbound public API-key system.** A third party cannot call
  Qervon's API directly with their own issued API key today — every
  caller authenticates as a real user/tenant member (QAS-000004). There
  is a single static `QERVON_API_ACCESS_TOKEN` for internal
  machine-to-machine use, not a per-integration key-issuance system.
- **No webhook retry-with-backoff policy documented** beyond the
  worker's generic 5-minute reclaim window (QFS-000011) — a webhook
  delivery that keeps failing is retried the same way any other stuck
  outbox item is, with no delivery-specific backoff curve or a
  "disable after N consecutive failures" circuit breaker.
- **No SDK-generated client** for third parties beyond the
  hand-maintained, currently-empty `sdk/typescript/` placeholder (see
  QES-000005).

## References

- [QFS-000011](QFS-000011-scheduler.md) (the scheduler/outbox mechanism delivery uses), [QAS-000005](../qas/QAS-000005-api-integration-standard.md)
  (API conventions), [QES-000014](../qes/QES-000014-secure-coding-standard.md) (secure coding — the HTTPS-only/secret-hash
  validation rules above).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real outbound-webhook implementation and the explicit absence of inbound integration. |
