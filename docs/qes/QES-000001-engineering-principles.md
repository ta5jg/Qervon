<!-- =============================================================================
File:           docs/qes/QES-000001-engineering-principles.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The cross-language principles every Qervon codebase (Rust, Kotlin,
  Swift, JS) is expected to follow.

Specification:
  QAS-000001, QMI-000001.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000001 — Engineering Principles

## 1. Never fabricate

If a screen or a document describes a capability, it must be real and
checkable, or explicitly marked as not yet built (see QAS-000001's
honesty policy). This applies to UI ("does this button actually do
something?"), to numbers ("is this balance from a real API call?"), and
to documentation ("does this ADR describe what's actually in the code?").

## 2. Every file carries a standard header

Every source file (Rust, Kotlin, Swift, JS/TS, YAML, Markdown) in this
repository starts with a comment block:

```text
File:           <path from repo root>
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   <YYYY-MM-DD>
Version:        <semver-ish, bumped on real content changes>

Description:
  <what this file is for, one short paragraph>

Specification:
  <QAS-/QES-/QLS-/QMI-/ADR- IDs this file implements or is governed by>

License:
  Qervon License v1.0 — see LICENSE in the repository root.
```

This is what makes it possible to trace any file back to the governance
document that explains *why* it exists — see QES-000009.

## 3. Domain invariants live in one place

A business rule (an order can't skip from `Pending` to `Delivered`, a
fare is always computed server-side) is enforced in exactly one function,
called from every code path that could otherwise violate it — never
duplicated across a handler and a client-side check. See QAS-000001's
layering discussion.

## 4. Prefer the compiler/type system over runtime checks

Rust's ownership/borrow checker, Kotlin's null-safety, Swift's optionals
— all three languages chosen for this project (ADR-000001–ADR-000003)
give strong compile-time guarantees; code should be written to make
illegal states unrepresentable where the type system allows it, rather
than defending against them with runtime `if`-checks alone.

## 5. Real, working software over polished-looking placeholders

Given a choice between a smaller feature that is fully real end-to-end,
and a larger one that is partially faked, build the smaller real one.
This is the principle behind, e.g., choosing a manual "QR verified"
checkbox over a fake camera-scan animation on a platform (browser) where
a real scan wasn't implemented — see QAS-000008.

## References

- [QAS-000001](../qas/QAS-000001-architecture-philosophy.md) (architecture philosophy, the fuller version of #1/#3),
  [QES-000002](QES-000002-rust-engineering-standard.md) through [QES-000005](QES-000005-typescript-react-standard.md) (per-language application of these
  principles), [QES-000009](QES-000009-documentation-standard.md) (documentation standard, the header format).

---

## Revision History

| Version | Date | Description |
| --------- | ------ | ------------- |
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real, actually-followed cross-language principles. |
