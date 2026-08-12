<!-- =============================================================================
File:           docs/qfs/QFS-000008-policy-engine.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  No generic, data-driven policy engine exists. Authorization is real,
  hardcoded RBAC checks — see QFS-000009 for the fuller detail.

Specification:
  QAS-000004, QAS-000011, QFS-000009.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QFS-000008 — Policy Engine

**Status: Vision / Not Implemented** (as a generic engine). See
QFS-000009 for what actually exists (hardcoded RBAC).

## What "policy engine" would mean

A system where authorization rules are expressed as data (e.g. an
OPA/Rego-style policy language, or a database-stored rule set) and
evaluated by a shared interpreter for every request — so "who can do
what" can change without a code deploy, and can express conditions more
complex than "does this role have access to this route."

## What actually exists instead

Rust middleware functions (`require_operational_access`,
`require_courier_access`, `require_signed_user`, ...) hand-written per
route group, plus explicit ownership checks in individual handlers
(`require_customer_order`, `require_courier_order`) — see QFS-000009 and
QAS-000004 for the real detail. Every authorization rule is compiled Rust
code; changing one requires a code change and redeploy.

## Why this hasn't been generalized

The current rule set (role-based route access + tenant/ownership checks)
is small and stable enough that hand-written middleware is both simpler
to audit and gives stronger compile-time guarantees (a typo in a policy
DSL fails at runtime; a typo in a Rust middleware function usually fails
to compile) than a generic engine would. Revisit if/when authorization
rules need to vary per-tenant in ways a fixed set of Rust functions can't
express (e.g. a tenant-configurable approval workflow).

## References

- QFS-000009 (permission engine — the real RBAC implementation),
  QAS-000004 (security architecture), QAS-000011 (multi-tenant
  enforcement, the other half of "who can access what").

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as an explicit Vision/Not Implemented status pointing to the real RBAC in QFS-000009. |
