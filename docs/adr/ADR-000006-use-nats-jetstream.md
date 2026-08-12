<!-- =============================================================================
File:           docs/adr/ADR-000006-use-nats-jetstream.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Architecture Decision Record: NATS JetStream was planned as the
  cross-service event bus but was never adopted. The system instead uses
  PostgreSQL LISTEN/NOTIFY plus an in-process Tokio broadcast channel for
  its one real streaming use case (live courier location).

Specification:
  QMI-000000, QAS-000003.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# ADR-000006 — Event Bus: NATS JetStream (Not Adopted)

- **Status:** Not Adopted — superseded by a simpler mechanism, described
  below.
- **Date:** original intent 2026-08-05; corrected 2026-08-12.
- **Deciders:** Irfan Gedik.

## Original Context and Intent

The source architecture PDFs describe a "hybrid event architecture"
(see ADR-000010) with NATS JetStream as the durable, cross-service event
bus connecting a future set of independently-deployable services
(Delivery, Fleet, Warehouse, Dispatch, ...).

## What Actually Exists

There is exactly one real, live streaming requirement implemented today:
propagating a courier's location update to whoever is watching that
courier's order (the admin dashboard's live map, a customer's tracking
screen). This is implemented with two much simpler primitives, no message
broker at all:

- **PostgreSQL `pg_notify`** (`SELECT pg_notify('qervon_location_updates', $1)`,
  see `backend/apps/api-gateway/src/state.rs`) fires whenever a location
  update is persisted to PostgreSQL, for durability across multiple
  `api-gateway` instances sharing one database.
- **An in-process `tokio::sync::broadcast` channel** fans that same event
  out to every open `/ws/tracking` WebSocket connection on that instance
  without a second network hop.

There is no NATS server anywhere in this codebase's dependency tree, no
JetStream stream/consumer configuration, and no multi-service deployment
today for an event bus to connect — `api-gateway` is presently the only
network-facing binary (see ADR-000007, modular monolith).

## Decision (Correcting the Record)

Do not adopt NATS/JetStream at this stage. Continue using
PostgreSQL `LISTEN`/`NOTIFY` + an in-process broadcast channel for the
one real streaming use case, until a genuine multi-service deployment or
a durable-replay requirement (e.g. event sourcing, cross-service sagas)
actually exists to justify the operational cost of running and monitoring
a message broker.

## Consequences

- **Positive:** one fewer stateful service to deploy, monitor, and secure
  on a budget single-VPS deployment; `pg_notify` piggybacks on a database
  connection the system already needs, so there's no additional network
  surface.
- **Negative:** `pg_notify` payloads are capped at 8000 bytes and are
  **not durable** — a notification sent while no one is listening is
  lost; this is acceptable for "best-effort live position updates" but
  would not be acceptable for financial events (which are instead written
  directly to the PostgreSQL ledger tables, not sent as fire-and-forget
  notifications). If Qervon ever needs guaranteed-delivery events between
  independently-deployed services, this decision should be revisited.
- **Neutral:** nothing here blocks introducing NATS JetStream later for a
  specific, concrete need — it is additive, not a rearchitecture, of the
  modular monolith's internals.

## Alternatives Considered

- **NATS JetStream** (originally planned): deferred — no current need for
  durable cross-service event replay, since there is currently one
  service.
- **Redis Pub/Sub** (mentioned in the source PDFs as a live-map option):
  would add a second stateful dependency for a problem `pg_notify` +
  in-process broadcast already solves at this scale.
- **Apache Kafka** (mentioned in the source PDFs "for very large
  systems"): explicitly out of scope at current scale; revisit only if
  event volume or replay/audit requirements genuinely demand it.

## References

- [QAS-000003](../qas/QAS-000003-event-architecture.md) (event architecture, rewritten to match this decision).
- [ADR-000007](ADR-000007-use-modular-monolith-first.md) (modular monolith), [ADR-000010](ADR-000010-use-hybrid-event-architecture.md) (hybrid event architecture).
- `backend/apps/api-gateway/src/state.rs` (the real `pg_notify` call site).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs, assuming NATS JetStream. |
| 0.2.0 | 2026-08-12 | Corrected: NATS was never adopted; documented the real pg_notify + broadcast-channel mechanism. |
