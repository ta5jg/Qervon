<!-- =============================================================================
File:           docs/qes/QES-000014-secure-coding-standard.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Concrete secure-coding rules, several derived directly from a real
  vulnerability found and fixed in this codebase.

Specification:
  QAS-000004, QES-000010, QES-000011.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000014 — Secure Coding Standard

## Rules derived from a real, found-and-fixed issue

- **Every user-controllable string rendered via `innerHTML` must be
  HTML-escaped.** This is not a hypothetical rule — a real XSS gap
  (an unescaped order-address label in `mobile-customer.html`) was found
  during the 2026-08-12 audit and fixed by applying the same
  `escapeHtml()` helper every other render path in the same file already
  used. New web-page code must follow this consistently, not
  case-by-case.
- **Pin every third-party CDN script to an exact version.** Also found
  during that audit (`lucide@latest` across four pages) — an unpinned
  CDN dependency executes with full page privileges if the "latest"
  build ever changes unexpectedly or the CDN is compromised.

## Authentication/secrets

- Passwords: `argon2` only, never a faster/weaker hash (see QAS-000004).
- Tokens: never logged, never placed in a URL query string, always
  `HttpOnly` where the client doesn't need to read them (see
  QAS-000004's cookie attribute table).
- Secrets (signing keys, webhook encryption keys, DB credentials):
  environment variables on the VPS, never committed to git, never
  echoed in a log line (see QAS-000014).

## Input validation

- Every mutating endpoint's request DTO is a typed `serde::Deserialize`
  struct (QAS-000005) — an unknown/malformed field either gets ignored
  (extra fields) or the whole request is rejected (missing required
  fields, wrong type), never partially processed.
- Redirect targets from user input (`return_to` query parameter on
  `/login`) must be validated to start with `/` and not `//` before use,
  to prevent an open-redirect/protocol-relative-URL attack — see
  `login.html`'s `returnTo` handling as the reference implementation.

## Multi-tenancy as a security boundary

Every resource-ownership check (QAS-000011) is a security control, not
just a data-correctness one — a bug here is a cross-tenant data leak, not
just a display glitch. New endpoints touching tenant-scoped data must
have a test asserting cross-tenant access is rejected, following the
existing pattern in `api_flow.rs`.

## What is not implemented

- No automated dependency vulnerability scanning (`security.yml` is an
  empty placeholder — see QES-000010/QES-000011).
- No WAF, no automated penetration testing.
- No formal secrets-rotation automation beyond the manual procedure in
  [docs/operations/key-rotation-runbook.md](../operations/key-rotation-runbook.md).

## References

- [QAS-000004](../qas/QAS-000004-security-architecture.md) (the security architecture these rules protect), [QES-000010](QES-000010-ci-cd-standard.md)
  (CI gaps — no automated scanning yet), [QES-000011](QES-000011-dependency-policy.md) (dependency policy).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with concrete rules, several directly derived from the real XSS/CDN-pinning fixes made this same day. |
