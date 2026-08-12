<!-- =============================================================================
File:           docs/qfs/QFS-000015-meta-platform-roadmap.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  A roadmap, by definition forward-looking — collects every "revisit
  when a real need exists" pointer scattered across the QFS series in
  one place.

Specification:
  QFS-000001 through QFS-000014.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QFS-000015 — Meta-Platform Roadmap

**Status: Vision/roadmap document by definition** — nothing here is a
claim about the present; it collects the concrete "revisit this when X"
triggers already stated throughout the QFS series, in one place, so the
condition for doing each is explicit rather than "someday."

## Candidate future work, with its real trigger condition

| Idea | Trigger condition (from) |
| --- | --- |
| Generic module-plugin mechanism | A genuine need for third-party or dynamically-loaded extensibility beyond the current internally-maintained `modules/*` crates (QFS-000005) |
| Generic workflow engine | A third or fourth genuinely distinct hand-built state machine appears, making the duplication real rather than theoretical (QFS-000007) |
| Generic policy/rule engine | Authorization or pricing/scoring rules need to vary per-tenant in ways fixed Rust functions can't express (QFS-000008, QFS-000010) |
| Separate AI-serving component | A real ML model (with different resource/scaling needs than the rest of the API) replaces the current deterministic heuristics (QFS-000013, QAS-000009) |
| Message broker (NATS or otherwise) | A genuine multi-service deployment or a durable-replay/event-sourcing requirement emerges (ADR-000006, ADR-000010) |
| PostGIS adoption | A "couriers within N km" spatial-index query becomes necessary at the courier counts this system runs at (ADR-000005) |
| Multi-region/failover infrastructure | Uptime requirements exceed what a single VPS with manual backup/restore can provide (QAS-000015) |
| Real CI for mobile + fixing the fake `mobile-build.yml` step | Immediately actionable, not gated on any future condition — this is a known, already-real gap (QES-000010) |
| React (or other) web platform rewrite | A concrete requirement for component reuse/type-checked API client across pages emerges that the current per-page vanilla JS can't reasonably satisfy (ADR-000004) |
| Object storage for delivery-photo upload | Before `photo_evidence_url` can be filled with a real URL from either mobile app (QLS-000013) |

## What this document is not

Not a committed schedule, not a prioritized backlog with dates —
[BACKEND_BACKLOG.md](../../BACKEND_BACKLOG.md) is the actual current,
maintained backlog with concrete near-term items. This document is the
longer-horizon "what would make the QFS series' vision documents
real" reference.

## References

- Every QFS-000002 through QFS-000014 document (each states its own
  trigger condition in more detail than the table above), BACKEND_BACKLOG.md.

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as a real collection of trigger conditions gathered from across the QFS series. |
