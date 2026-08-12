<!-- =============================================================================
File:           docs/qas/QAS-000001-architecture-philosophy.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The architectural principles that hold across the whole system:
  layering, module boundaries, and the project's honesty policy.

Specification:
  QMI-000000, ADR-000001, ADR-000007.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000001 — Architecture Philosophy

**Status: Implemented.**

## Clean/DDD layering

The backend follows Domain-Driven Design with a strict dependency
direction: `domain` → `application` → `infrastructure`, with
`api-contracts` as a leaf shared by the HTTP layer only.

- `crates/domain`: entities and value objects with their own invariants
  (`Order`, `Courier`, `CustomerProfile`, `DeliveryPricing`, `Money`,
  `Location`, ...) and pure business logic (state-transition methods like
  `Order::assign`, fare-quoting math). No knowledge of HTTP or SQL.
- `crates/application`: use-case services (`OrderService`,
  `DispatchService`, `PricingService`, `CourierWalletService`, ...) that
  orchestrate domain objects and repository traits. No knowledge of Axum
  or `sqlx` concretely — only the repository traits domain/application
  defines.
- `crates/infrastructure`: implements those repository traits twice —
  once in-memory (fast local dev, unit tests), once against PostgreSQL
  (`sqlx`) — with no code duplication in domain/application.
- `modules/*`: thin façades over application services, one per bounded
  context, that `apps/api-gateway` composes.

The rule this enforces: a domain invariant (e.g. "an order cannot move
from `Delivered` back to `Pending`") lives in exactly one place
(`qervon_domain::order`) and cannot be bypassed by adding a new HTTP
handler that skips it — every code path to mutate an order goes through
the same domain method.

## Modular monolith, not microservices

See ADR-000007. One deployable binary, many internal crate boundaries.

## Native-first for clients, not a shared UI layer

Both mobile platforms (ADR-000002, ADR-000003) and the web layer
(ADR-000004) are built against the same backend HTTP contract but with
zero shared UI/business-logic code between them — each platform gets a
real, idiomatic implementation rather than a compromise cross-platform
layer. The tradeoff (duplicated screen code across iOS/Android/web) is
accepted deliberately; see each ADR's Consequences section.

## The honesty policy (the principle that shaped this rewrite)

This is the philosophy that most concretely shaped how this documentation
set itself was rewritten, and it applies equally to code:

- **Never fabricate data or a working feature.** If a screen shows a
  number, it came from a real API call, or it is visibly marked as an
  estimate/placeholder. Concretely: the web pages' "Kazanç" cards used to
  show a hardcoded `₺850.00`; the fix was to call the real
  `GET /v1/courier/me/wallet` endpoint, not to make the number look more
  plausible.
- **A missing capability is stated, not hidden.** If something from the
  source vision PDFs isn't built, the relevant document says so plainly
  (see QMI-000000's Vision/Not Implemented status) rather than describing
  it as if it exists.
- **A decision that changed is recorded, not erased.** See ADR-000004 and
  ADR-000006 — both record what was originally planned and what actually
  happened, so the history stays legible.

## References

- ADR-000001 (Rust), ADR-000007 (modular monolith), QAS-000002 (domain
  model, the concrete result of this layering).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real layering rules and the project's honesty policy. |
