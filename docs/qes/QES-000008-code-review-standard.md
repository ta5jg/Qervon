<!-- =============================================================================
File:           docs/qes/QES-000008-code-review-standard.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  What a reviewer is expected to check before approving a change.

Specification:
  QES-000001, QES-000007.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000008 — Code Review Standard

## What every review must check

1. **Does it match QES-000001's honesty principle?** No fabricated data,
   no UI element that looks functional but isn't, no documentation claim
   that outpaces the actual implementation.
2. **Are domain invariants enforced in one place** (QAS-000001), not
   duplicated or, worse, only checked client-side?
3. **Is there a passing test for the happy path and at least one
   rejection path** for new backend behavior (QES-000006)?
4. **Does every new/changed file carry the standard header** (QES-000009)
   with an accurate `Specification:` line pointing at the governance
   document(s) that actually govern it?
5. **If this change makes an existing governance document's claims
   wrong, is that document updated in the same change** (QMI-000001) —
   not deferred to a "docs follow-up" that historically never happened
   (see QMI-000000's honesty-policy background on why this rule exists).

## Reviewer tooling available (real)

- The `bugbot` and `security-review` Cursor subagent skills can be
  invoked to review a diff for defects/security issues respectively —
  used as a supplement to human review, not a replacement for it.
- `.github/CODEOWNERS` routes review to the right person by path.

## What is not automated

There is no required-approvals branch-protection rule documented, no
automated linting-as-a-bot-comment on PRs beyond what CI's pass/fail
already communicates.

## References

- QES-000001 (the principles this enforces), QES-000007 (git workflow),
  QES-000009 (the header format reviewers check for).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real review checklist this project actually applies. |
