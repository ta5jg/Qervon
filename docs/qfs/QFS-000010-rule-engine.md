<!-- =============================================================================
File:           docs/qfs/QFS-000010-rule-engine.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  No generic, data-driven rule engine exists. Pricing/coupon/dispatch
  scoring rules are all hardcoded Rust formulas.

Specification:
  QAS-000009, QLS-000009, QFS-000007.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QFS-000010 — Rule Engine

**Status: Vision / Not Implemented** (as a generic engine).

## What "rule engine" would mean

A system where business rules (fare formulas, coupon eligibility, AI
Dispatcher scoring weights) are expressed as configurable data — evaluated
by a shared interpreter — rather than compiled Rust, so an operator could
adjust, say, the dispatcher's vehicle-type weighting or a tenant's fare
formula without a code deploy.

## What actually exists instead — every "rule" is a Rust formula

- **Fare pricing** (`PricingService::quote_fare`, QLS-000002): per-tenant
  `DeliveryPricing { base_fare, per_km_rate, min_fare }` — this is the
  *one* place with real per-tenant configurability (via `GET`/`PUT
  /v1/pricing`), but the formula shape itself (base + per-km, floor at a
  minimum) is fixed Rust code, not a rule an operator could redefine.
- **Coupon discounts** (`PromoCouponEngine::apply_to_fare`, QLS-000009):
  a fixed set of discount kinds (percentage/flat), hardcoded in Rust.
- **AI Dispatcher scoring** (`calculate_score`, QAS-000009): the
  `0.7`/`0.3` ETA/distance weighting and per-vehicle-type multipliers are
  literal constants in the function, not configurable at all — not even
  per-tenant.

## Why this hasn't been generalized

Pricing has real, if narrow, per-tenant configurability already
(QLS-000009); the dispatcher-scoring weights have never needed to vary
per-tenant so far. Building a generic rule engine for two or three fixed
formulas would be speculative complexity with no current requirement
driving it (see QES-000001's principle on this).

## References

- QLS-000009 (billing/pricing — the one place with real
  configurability), QAS-000009 (AI Dispatcher — the hardcoded scoring
  formula), QFS-000007/QFS-000008 (the sibling "no generic engine"
  documents for workflow/policy).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as an explicit Vision/Not Implemented status listing every real "rule" as hardcoded Rust. |
