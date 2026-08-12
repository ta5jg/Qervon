<!-- =============================================================================
File:           docs/qas/QAS-000009-ai-architecture.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The real AI Dispatcher, Dynamic ETA, and AI Fraud Guard implementations
  — deterministic heuristics, not machine-learning models.

Specification:
  QAS-000002, QAS-000003, QLS-000003.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000009 — AI Architecture

**Status: Implemented — as deterministic heuristics, not ML models.**
Source: `backend/crates/application/src/ai_dispatcher.rs`.

## Honesty note on the word "AI"

Nothing in this system trains or runs a machine-learning model. "AI
Dispatcher", "AI ETA", and "AI Fraud Guard" are the source PDFs' names for
three real, deterministic, hand-written scoring functions. This document
uses those names because they're the names used throughout the code and
the other governance documents, but the underlying implementation is
plain arithmetic, testable and auditable line-by-line — not a black box.

## AI Dispatcher (`AiDispatcher::rank_candidates`/`calculate_score`)

For each available courier with a known current location:

```text
score = (eta_minutes * 0.7 + distance_km * 0.3) * vehicle_weight
```

where `vehicle_weight` is `1.0` for motorcycle, `1.2` for car, and for
bicycle either `0.9` (under 3 km — bicycles are favored for very short
hops) or `2.0` (3 km or more — heavily penalized for longer distances).
Couriers are ranked ascending (lowest score = best candidate) and the
best-ranked available courier is offered the job (see QLS-000003 for the
offer/accept/reject flow this feeds).

## Dynamic ETA (`AiDispatcher::calculate_dynamic_eta`)

```text
effective_speed = base_speed_kmh / (traffic_multiplier * weather_multiplier)
eta_minutes = (distance_km / effective_speed) * 60
```

Base speeds: bicycle 15 km/h, car 25 km/h, motorcycle 35 km/h.
`TrafficContext { congestion_multiplier, weather }` supports a traffic
multiplier and a weather multiplier (rainy = 1.25×, snowy = 1.60×) —
**but every real call site passes `None` for this context.** There is no
live traffic or weather data source wired in; the multiplier machinery
exists in code and is unit-tested, but production ETAs today are always
the base vehicle-speed calculation with no traffic/weather adjustment.
Wiring a real traffic/weather API is a concrete, scoped future
improvement, not a rewrite — the function signature already accepts it.

This same function powers both the AI Dispatcher's internal scoring and
the customer-facing `GET /v1/customer/orders/{id}/eta` endpoint.

## AI Fraud Guard (`AiDispatcher::detect_gps_fraud`)

```text
speed_kmh = (distance_km_between_two_consecutive_samples / elapsed_seconds) * 3600
is_fraudulent = speed_kmh > 160.0
risk_score = min(speed_kmh / 200.0, 1.0)
```

Called on every `POST /v1/courier/me/location`/`POST
/v1/couriers/{id}/location` write, comparing the new sample against the
courier's immediately-preceding recorded location. Flag-and-accept, not
block-and-reject: a flagged sample is still stored (with
`fraud_flagged=true`, `fraud_risk_score`), still updates the courier's
live position, but is visibly marked as suspicious to anyone viewing the
live map (a red marker, a warning banner) — see `index.html`'s AI
Fraud Guard tab. Nothing currently escalates a repeated fraud flag into
an account action (suspension, alert to an operator) — that response is
left to a human reviewing the dashboard today.

## What is not implemented

- **AI Route** (multi-stop route optimization for a courier carrying
  several orders) — mentioned in the source PDFs, not built; there is no
  batching of multiple orders onto one courier run today.
- **Any actual machine-learning component** — no training pipeline, no
  model file, no inference framework dependency exists in this codebase.

## References

- QAS-000002 (the `Courier`/`Location` types these functions operate on),
  QAS-000003 (the location-update pipeline this feeds), QLS-000003
  (dispatch domain, the offer flow this scoring drives).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real scoring formulas and the honest note that traffic/weather context is unused in practice. |
