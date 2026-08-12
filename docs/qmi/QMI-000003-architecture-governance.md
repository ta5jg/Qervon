<!-- =============================================================================
File:           docs/qmi/QMI-000003-architecture-governance.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  When a change requires a new or superseding ADR, and how architectural
  changes flow through QAS/QFS documents.

Specification:
  QMI-000000, QMI-000001.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QMI-000003 — Architecture Governance

## When a change needs a new ADR

Write a new ADR (see the template in `docs/adr/README.md`) when a change:

- Introduces a new language, framework, database, or major dependency
  not already covered by an existing ADR (e.g. adopting a message broker
  would need a new ADR even though ADR-000006 explains why one isn't used
  today).
- Reverses or materially narrows a prior decision — in which case the
  **old** ADR's status changes to Superseded/Not Adopted with a pointer
  forward, and a new ADR (or a rewrite of the old one, as done for
  ADR-000004 and ADR-000006 in this pass) records what actually happened
  and why.
- Changes a cross-cutting property everyone downstream relies on (the ID
  scheme, the multi-tenancy model, the auth token format).

A change does **not** need a new ADR for: adding a new field to an
existing DTO, adding a new endpoint that follows an existing pattern,
adding a new test, or any change fully described by an existing QAS/QES
document.

## When a QAS/QFS document needs updating vs. a new ADR

- **ADR**: records the *decision and why*, once, immutably (append status
  changes, never delete history).
- **QAS/QFS**: records the *current shape* of a subsystem, and is
  expected to be kept continuously accurate as the subsystem evolves —
  unlike an ADR, a QAS document should describe today's system, not a
  point-in-time decision.

Concretely: choosing PostgreSQL was an ADR (ADR-000005). How the
migration system is structured, which schemas exist, and how the
memory/PostgreSQL dual-backend split works is QAS-000006, and that
document should be edited directly (with a revision-history entry) every
time the persistence layer changes shape, without needing a new ADR each
time.

## Review

Currently self-reviewed by the sole maintainer. As the team grows, the
recommendation is: any ADR needs at least one other engineer's sign-off
before merging (an architectural decision made in isolation is exactly
the failure mode ADRs exist to prevent); QAS/QES/QLS edits can be
reviewed like any other pull request.

## References

- QMI-000000 (master index), QMI-000001 (document governance),
  `docs/adr/README.md` (the ADR index and template).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with real rules distinguishing ADRs from QAS/QFS maintenance. |
