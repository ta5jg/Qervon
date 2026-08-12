<!-- =============================================================================
File:           docs/qas/QAS-000003-event-architecture.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  How live courier location events flow from a location POST to every
  interested viewer — the one real streaming path in this system.

Specification:
  ADR-000006, ADR-000010.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000003 — Event Architecture

**Status: Implemented** (narrowly — see ADR-000010 for why this is
deliberately not a general-purpose event bus).

## The one real event pipeline: live location

1. A courier's client (native app or `mobile-courier.html`) calls
   `POST /v1/courier/me/location` with `{latitude, longitude, speed_kmh,
   battery_pct}`.
2. `TrackingService` (application layer) runs the sample through the AI
   Fraud Guard's speed-anomaly check (see QAS-000009), then persists it
   — either to the in-memory recent-location cache, or to PostgreSQL.
3. On the PostgreSQL path, the same write additionally calls
   `SELECT pg_notify('qervon_location_updates', <json payload>)`.
4. Inside the `api-gateway` process, a background task subscribed to that
   Postgres channel republishes each notification onto an in-process
   `tokio::sync::broadcast` channel.
5. Every open `GET /ws/tracking` (admin) or `GET /ws/tracking/customer`
   WebSocket connection, and every polling `GET /v1/orders/{id}/tracking`
   /`GET /v1/tracking/live` request, reads from that same in-memory state.

This gives multiple independent viewers (an admin's live map, one or more
customers each tracking their own order) a consistent, near-real-time
view without a message broker, and without every viewer hitting
PostgreSQL directly on every poll.

## Everything else is request/response, not events

Order creation, dispatch assignment/offer/accept/reject, delivery
completion, wallet crediting, coupon redemption — all of these are
synchronous service calls inside a single HTTP request/response, with
normal SQL transactions for atomicity. See ADR-000010 for why this split
is deliberate.

## Delivery guarantees

- **Location events:** best-effort, not durable. A client that is not
  currently connected to a WebSocket or polling misses events published
  while it was disconnected — acceptable for "where is the courier right
  now", unacceptable for anything financial (which is why finance never
  flows through this pipeline).
- **Everything else:** ACID within a single PostgreSQL transaction per
  request, no distributed-transaction/saga machinery.

## References

- [ADR-000006](../adr/ADR-000006-use-nats-jetstream.md) (why NATS was not adopted), [ADR-000010](../adr/ADR-000010-use-hybrid-event-architecture.md) (the hybrid
  decision), [QAS-000009](QAS-000009-ai-architecture.md) (AI Fraud Guard, which sits in this pipeline).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten to describe the real pg_notify + broadcast-channel pipeline. |
