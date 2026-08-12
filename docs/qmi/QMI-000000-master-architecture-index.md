<!-- =============================================================================
File:           docs/qmi/QMI-000000-master-architecture-index.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Master index of Qervon's governance documentation: what each document
  series covers, how they relate, and where to start.

Specification:
  QMI-000001, QMI-000002, QMI-000003.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QMI-000000 — Master Architecture Index

## How to read this documentation set

Qervon's `docs/` tree has six series, each with a distinct job. Read this
page first; it tells you which series to open for a given question.

| Series | Full name | Answers the question | Count |
| --- | --- | --- | --- |
| **QMI** | Qervon Meta-Index | "How is this documentation itself organized and governed?" | 4 |
| **ADR** | Architecture Decision Record | "Why did we choose X over Y, and is that decision still in force?" | 10 |
| **QAS** | Qervon Architecture Specification | "How is subsystem X actually built?" | 15 |
| **QES** | Qervon Engineering Standard | "What conventions do we follow when writing code/tests/PRs?" | 15 |
| **QLS** | Qervon Logistics Specification | "What does domain concept X (an Order, a Courier, ...) actually mean and do?" | 15 |
| **QFS** | Qervon Foundation Specification | "Is there a generic extensible platform/kernel underneath all this?" (see honesty note below) | 15 |

## Honesty policy for this documentation set (read this)

Every one of these 74 files originally contained nothing but a repeated
auto-generated dump of the two source vision PDFs (`docs/qervon-1.md`,
`docs/qervon-2.md`) behind the file's title — regardless of topic. As of
2026-08-12 this was corrected file-by-file. Each rewritten document now
carries an explicit status:

- **Implemented** — describes something real, checkable against the
  actual source tree (a file path, a crate, a test). This is the default
  and most common status for QAS, QES, ADR.
- **Vision / Not Implemented** — describes something from the source
  PDFs that does not exist in the codebase today, clearly labeled as
  such with an explanation of what (if anything) exists instead. Most of
  **QFS** falls here — see QFS-000001 for why — along with a handful of
  QLS domains (see QLS-000011, QLS-000012).
- **Superseded** — a decision that was made and then changed; the record
  is kept (not deleted) with a pointer to what replaced it (see
  ADR-000004, ADR-000006).

If a document you're reading does not carry one of these statuses near
the top, treat it as stale and flag it — the policy is that every file
should.

## Where to actually start

- Building the backend? Start at QAS-000001 (architecture philosophy),
  then QAS-000002 (domain model), then the relevant QLS domain document.
- Building a mobile screen? Start at QAS-000007 (mobile platform
  architecture), then ADR-000002/ADR-000003 for the per-platform
  decision.
- Touching the web pages? Start at QAS-000008 and ADR-000004.
- Writing code and want the house style? Go straight to the matching
  QES document for your language.
- Deploying? QAS-000014 (deployment architecture) and
  [docs/operations/deployment-runbook.md](../operations/deployment-runbook.md).

## References

- [QMI-000001](QMI-000001-document-governance.md) (document governance — how these files are maintained),
  [QMI-000002](QMI-000002-versioning-policy.md) (versioning policy), [QMI-000003](QMI-000003-architecture-governance.md) (architecture governance —
  when a change needs an ADR).
- Root [README.md](../../README.md), [BACKEND_BACKLOG.md](../../BACKEND_BACKLOG.md).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as a real navigational index with an explicit honesty/status policy. |
