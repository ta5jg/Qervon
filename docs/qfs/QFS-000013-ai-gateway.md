<!-- =============================================================================
File:           docs/qfs/QFS-000013-ai-gateway.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  No separate "AI Gateway" service exists. AI logic runs in-process as
  plain function calls — see QAS-000009.

Specification:
  QAS-000009, ADR-000007, QFS-000001.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QFS-000013 — AI Gateway

**Status: Vision / Not Implemented** (as a separate service). Real AI
logic exists, in-process — see QAS-000009.

## What "AI Gateway" would mean

A dedicated service (or at minimum a distinct internal API boundary)
that all AI-related requests (dispatch scoring, ETA, fraud detection)
route through — implying it could be scaled, deployed, or even swapped
out (e.g. for a real ML-model-serving backend) independently of the main
API.

## What actually exists instead

`AiDispatcher` (`backend/crates/application/src/ai_dispatcher.rs`) is a
plain Rust struct with plain functions, called directly by
`DispatchService` and by the relevant HTTP handlers — in the same
process, same request, no network hop, no separate deployable. See
QAS-000009 for the actual scoring/ETA/fraud formulas.

## Why this hasn't been separated out

There is no current need to scale or deploy AI logic independently — it
is deterministic arithmetic (QAS-000009), not a resource-intensive model
inference call that would benefit from its own scaling story. Separating
it into its own service now would add a network hop and an operational
unit with no corresponding benefit. Revisit if/when a real ML model
(with meaningfully different resource/scaling needs than the rest of the
API) is actually introduced.

## References

- [QAS-000009](../qas/QAS-000009-ai-architecture.md) (the real AI Dispatcher/ETA/Fraud Guard implementation),
  [ADR-000007](../adr/ADR-000007-use-modular-monolith-first.md) (modular monolith — the same reasoning applies here).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as an explicit Vision/Not Implemented status pointing to the real in-process AI logic in QAS-000009. |
