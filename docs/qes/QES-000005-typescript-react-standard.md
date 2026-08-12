<!-- =============================================================================
File:           docs/qes/QES-000005-typescript-react-standard.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Honest status check: no TypeScript or React code currently exists
  anywhere in this repository. This document records why, and what
  standard would apply if/when either is actually adopted.

Specification:
  ADR-000004, QAS-000008.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000005 — TypeScript/React Standard

**Status: Vision / Not Currently Applicable.** There is no TypeScript or
React code in this repository today.

## Why this document has nothing to enforce right now

- The web platform is vanilla HTML/CSS/JS (see ADR-000004, QAS-000008) —
  not TypeScript, not React. This was a deliberate decision: an already
  scaffolded React/TypeScript directory (`web/`) contained zero real
  code and was deleted rather than built out, in favor of the vanilla
  pages that already worked.
- `sdk/typescript/` (a *client SDK* for third parties integrating with
  Qervon's API, conceptually unrelated to the web platform) is, as of
  this writing, in the same state the deleted `web/` scaffold was in: a
  `package.json` containing only a license-header comment, no real
  dependencies, no `src/` implementation beyond an empty directory. It
  has not been addressed in this documentation pass — see
  BACKEND_BACKLOG.md if/when it is picked up.

## If TypeScript/React is adopted later

Should either the web platform be rebuilt in React (a large undertaking
— see ADR-000004's Consequences) or `sdk/typescript/` be built out for
real, this document should be rewritten (not left as a placeholder) with
actual enforced rules: strict `tsconfig` (`strict: true`, no implicit
`any`), an actual linter/formatter configuration that CI enforces (see
QES-000010's note that `.github/workflows/web.yml` is currently an empty
placeholder with no jobs), and a real testing framework choice. Writing
those rules speculatively, before any real code exists to apply them to,
would repeat the exact mistake this whole documentation-rewrite pass
exists to fix — see QMI-000000's honesty policy.

## References

- [ADR-000004](../adr/ADR-000004-use-react-typescript-for-web.md) (the decision this document's status follows from),
  [QAS-000008](../qas/QAS-000008-web-platform-architecture.md) (the real web architecture), [QES-000010](QES-000010-ci-cd-standard.md) (CI/CD standard,
  noting `web.yml`'s placeholder state).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs, assuming a React/TypeScript web platform. |
| 0.2.0 | 2026-08-12 | Rewritten as an explicit "Vision / Not Currently Applicable" status matching ADR-000004. |
