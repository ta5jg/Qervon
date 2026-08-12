<!-- =============================================================================
File:           docs/qmi/QMI-000001-document-governance.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Rules for who can change governance documents, how, and what review
  they need.

Specification:
  QMI-000000.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QMI-000001 — Document Governance

## Ownership

There is currently one maintainer (Irfan Gedik / USDTG GROUP TECHNOLOGY
LLC) for all of `docs/`. As the team grows, ownership should be assigned
per series (e.g. a backend lead owns QES-000002, a mobile lead owns
QAS-000007), recorded here.

## When a document must be updated

A governance document is **stale** — and must be updated in the same
change that causes the staleness — when:

- A file path, crate name, module name, or endpoint it references is
  renamed, moved, or removed.
- An ADR it depends on is superseded (see QMI-000003).
- A "Vision / Not Implemented" section it describes gets actually built —
  the status must flip to "Implemented" and the description corrected to
  match the real implementation, not just have its status word changed.

In practice: if a change to the mobile apps, backend, or web pages makes
a governance document's factual claims wrong, fixing the document is part
of that change, not a separate follow-up ticket that may never happen —
this is exactly the failure mode that produced 74 files of unchanged PDF
placeholder text over the life of this project so far.

## Format requirements

Every governance document must have:

1. The standard file header comment (see any file in this series for the
   exact fields — File/Project/Author/Developer/Created Date/Version/
   Description/Specification/License).
2. An explicit status marker near the top for ADRs (Accepted/Superseded/
   Not Adopted) or an explicit "Vision / Not Implemented" callout for
   QAS/QFS/QLS documents describing something not yet built (see
   QMI-000000's honesty policy).
3. A `# References` section linking related documents by ID.
4. A `# Revision History` table at the bottom, appended to (never
   rewritten) on each substantive change.

## What does not belong in a governance document

- Marketing language or unqualified superlatives ("industry-leading",
  "best-in-class") — these documents are read by engineers who need
  facts, not by prospective customers.
- Untagged aspirational claims — anything not yet built must say so.
- Duplicated content that belongs in code comments or a README instead
  — governance documents describe *why* and *what the contract is*, not
  line-by-line *how*; the code and its own comments are the source of
  truth for implementation detail.

## References

- [QMI-000000](QMI-000000-master-architecture-index.md) (master index and honesty policy), [QMI-000002](QMI-000002-versioning-policy.md) (versioning),
  [QMI-000003](QMI-000003-architecture-governance.md) (architecture governance / ADR process).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with real governance rules, motivated by this rewrite pass itself. |
