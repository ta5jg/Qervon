<!-- =============================================================================
File:           docs/qes/QES-000015-accessibility-standard.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Honest state of accessibility work: a few real ARIA attributes exist
  on the web pages; there is no systematic accessibility testing on any
  platform.

Specification:
  QAS-000007, QAS-000008.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000015 — Accessibility Standard

**Status: Implemented (partial, web only) — no systematic testing on
any platform.**

## What exists

The shipped web pages use some real ARIA attributes where a screen
reader would otherwise miss a dynamic update: status/error message
regions use `role="status" aria-live="polite"` (e.g. `mobile-customer.html`'s
`#order-message`, `#live-status`), and a handful of form inputs have
explicit `aria-label`s where a visual `<label>` alone wouldn't associate
correctly (e.g. `customer-pickup`, `customer-dropoff` in `customer.html`).
This coverage is inconsistent across pages — it was added ad hoc, not
systematically.

Native mobile apps use each platform's default accessibility behavior
(SwiftUI/Compose both provide reasonable defaults — e.g. a `Text` label
is read by VoiceOver/TalkBack automatically) but neither app has had a
dedicated accessibility pass (custom `accessibilityLabel`s for icon-only
buttons, dynamic type/font-scaling verification, color-contrast
verification against WCAG).

## What is not implemented

- No automated accessibility testing (no `axe-core` run against the web
  pages, no Accessibility Scanner/Xcode Accessibility Inspector audit
  recorded for either mobile app).
- No documented minimum WCAG conformance target (A/AA/AAA) — none has
  been chosen yet.
- No dedicated color-contrast check against the dark-themed web pages'
  palette (see the CSS custom properties in `index.html`/`customer.html`)
  or the mobile design systems' color palettes
  (`QervonDesignSystem`/`core:designsystem`).

## Recommended next steps (not yet implemented)

1. Pick a concrete conformance target (WCAG 2.1 AA is a reasonable
   default) and audit the web pages against it.
2. Add `accessibilityLabel`/`contentDescription` to every icon-only
   button on both mobile apps (several exist in the bottom navigation
   bars — see QAS-000007).
3. Add an automated check (even a simple `axe-core` CI step against the
   locally-running `api-gateway`) so regressions are caught, rather than
   relying on manual review.

## References

- [QAS-000007](../qas/QAS-000007-mobile-platform-architecture.md) (mobile platform architecture), [QAS-000008](../qas/QAS-000008-web-platform-architecture.md) (web platform
  architecture) — both describe the UI surfaces this document assesses.

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with an honest inventory of the real (partial) ARIA coverage and the absence of systematic testing. |
