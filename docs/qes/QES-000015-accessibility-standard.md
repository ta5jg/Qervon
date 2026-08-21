<!-- =============================================================================
File:           docs/qes/QES-000015-accessibility-standard.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  WCAG 2.1 AA web target, automated axe-core regression gate, and the
  remaining native-device accessibility validation boundary.

Specification:
  QAS-000007, QAS-000008.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000015 — Accessibility Standard

**Status: Implemented for the web CI gate; native device audit remains.**

## What exists

The shipped web pages use some real ARIA attributes where a screen
reader would otherwise miss a dynamic update: status/error message
regions use `role="status" aria-live="polite"` (e.g. `mobile-customer.html`'s
`#order-message`, `#live-status`), and a handful of form inputs have
explicit `aria-label`s where a visual `<label>` alone wouldn't associate
correctly (e.g. `customer-pickup`, `customer-dropoff` in `customer.html`).
All shipped web entry pages are scanned automatically in CI with `axe-core`.
The release gate fails on serious or critical WCAG 2.1 A/AA violations.

Native mobile apps use each platform's default accessibility behavior
(SwiftUI/Compose both provide reasonable defaults — e.g. a `Text` label
is read by VoiceOver/TalkBack automatically) but neither app has had a
dedicated accessibility pass (custom `accessibilityLabel`s for icon-only
buttons, dynamic type/font-scaling verification, color-contrast
verification against WCAG).

## Automated web gate

`tools/web-accessibility/check.mjs` serves every shipped static page in an
isolated local HTTP server, opens each page in headless Chrome, injects
`axe-core`, and checks WCAG 2.0/2.1 A and AA rules. GitHub Actions runs the
gate on every relevant web change. The first gate also fixed missing select
labels and serious dark-theme contrast failures.

## Remaining native-device work

1. Add `accessibilityLabel`/`contentDescription` to every icon-only
   button on both mobile apps (several exist in the bottom navigation
   bars — see QAS-000007).
2. Record VoiceOver, TalkBack, Dynamic Type/font scaling, reduced-motion,
   and native color-contrast audits on physical devices.

## References

- [QAS-000007](../qas/QAS-000007-mobile-platform-architecture.md) (mobile platform architecture), [QAS-000008](../qas/QAS-000008-web-platform-architecture.md) (web platform
  architecture) — both describe the UI surfaces this document assesses.

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with an honest inventory of the real (partial) ARIA coverage and the absence of systematic testing. |
| 0.3.0 | 2026-08-21 | Set WCAG 2.1 AA target and added automated axe-core CI coverage for every shipped web page. |
