<!-- =============================================================================
File:           docs/adr/ADR-000010-use-hybrid-event-architecture.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Architecture Decision Record: most state changes are plain synchronous
  service calls with direct database writes; only live courier location
  is treated as a genuine event stream. This is the "hybrid" — mostly
  request/response, narrowly event-driven where it actually matters.

Specification:
  QMI-000000, QAS-000003.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# ADR-000010 — Hybrid Event Architecture (Narrow, Not Pervasive)

- **Status:** Accepted — implemented, narrower in practice than the
  source PDFs' framing.
- **Date:** 2026-08-05.
- **Deciders:** Irfan Gedik.

## Context

The source architecture PDFs describe an event-driven system broadly.
In practice, most of Qervon's state transitions — order creation, courier
assignment, delivery completion, wallet credit — are naturally
transactional: a customer's `POST /v1/customer/orders` call needs to
either fully succeed (order row + tenant binding + optional coupon
redemption) or fully fail, inside one request. Treating every one of
these as an asynchronous event with eventual consistency would add
complexity (idempotency keys, retry/dead-letter handling, event
ordering) with no benefit for something that must be strongly consistent
anyway.

## Decision

Use a **hybrid** approach:

- **Direct, synchronous service calls** (the application-layer services
  in `crates/application`, called straight from Axum handlers) for
  everything that needs strong consistency within a single request:
  order lifecycle transitions, dispatch assignment, wallet
  credits/debits, coupon redemption. These are plain Rust function calls
  and SQL transactions — "events" in name only, if at all.
- **A genuine publish/subscribe event stream** for the one case that
  actually benefits from it: live courier location updates, which many
  independent viewers (an admin's live map, a customer's tracking screen)
  need to receive as they happen, with no single request/response pair
  that could carry them. This uses PostgreSQL `LISTEN`/`NOTIFY` plus an
  in-process broadcast channel — see ADR-000006 for why NATS JetStream
  was evaluated and not adopted for this.

## Consequences

- **Positive:** the vast majority of the system stays simple to reason
  about (a request either succeeds or returns an error, no eventual
  consistency to think about); the one place that genuinely needs a
  stream (location) has one, sized appropriately (in-process +
  `pg_notify`, not a message broker).
- **Negative:** this is a narrower reading of "event-driven" than the
  source PDFs implied — there is no general-purpose domain event bus,
  no event sourcing, and no saga orchestration between bounded contexts.
  If a future requirement genuinely needs cross-context choreography
  (e.g., "when an order is delivered, asynchronously trigger three
  independent side effects that must not block the delivery response"),
  that would need new infrastructure, not a repurposing of the location
  pipeline.
- **Neutral:** this decision is compatible with introducing a real event
  bus later (see ADR-000006) if/when a concrete need for one exists.

## Alternatives Considered

- **Full event sourcing** (every state change is an appended event,
  current state derived by replay): rejected as substantial complexity
  with no concrete requirement driving it today.
- **Broad pub/sub for all state changes** (per the source PDFs' framing):
  rejected for the reasons in Context above — most changes need
  synchronous strong consistency, not eventual consistency.

## References

- [QAS-000003](../qas/QAS-000003-event-architecture.md) (event architecture), [ADR-000006](ADR-000006-use-nats-jetstream.md) (NATS JetStream, not
  adopted), [ADR-000007](ADR-000007-use-modular-monolith-first.md) (modular monolith).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs, framing this broadly. |
| 0.2.0 | 2026-08-12 | Rewritten to describe the actual narrow, hybrid implementation. |
