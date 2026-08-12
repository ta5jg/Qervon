<!-- =============================================================================
File:           docs/qfs/QFS-000009-permission-engine.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The real, hardcoded role-based access control (RBAC) — not a generic,
  data-driven permission engine.

Specification:
  QAS-000004, QAS-000011, QFS-000008.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QFS-000009 — Permission Engine

**Status: Implemented (as hardcoded RBAC) — no generic, data-driven
engine.**

## Roles

`UserRole`: `Customer`, `Company`, `Courier`, `Admin`, `SuperAdmin`,
`Operator`, `Dispatcher`, `FleetManager`, `Support`. Distinct from
`TenantMemberRole` (`Owner`, `Admin`, `Member`, ...), which governs
standing *within* a tenant rather than what kind of account it is (see
QAS-000011).

## Enforcement mechanism

Axum middleware functions, each checking the caller's role (from the
signed access-token claims, QAS-000004) against what a route group
requires:

- `require_operational_access` — admin/operator-level routes.
- `require_courier_access` — courier-only routes.
- `require_signed_user` — any authenticated user, role-agnostic.
- `require_location_publisher`, `require_tracking_consumer` — narrower
  checks for the location-update and tracking-read paths specifically.

Beyond role, explicit **ownership** checks
(`require_customer_order`, `require_courier_order`) verify the specific
resource belongs to the caller — a valid `Customer` role token does not
grant access to *every* order, only the caller's own (see QAS-000011).

## Why "hardcoded" rather than a generic engine

Every one of the checks above is a plain Rust function inspecting a
fixed set of fields (role, subject id, tenant id) — there is no
permission-rule data model, no admin UI to define new permission rules,
and no way to grant a custom, narrower permission to a specific user
without writing a new middleware function. See QFS-000008 for why this
hasn't been generalized into a data-driven policy engine.

## References

- [QAS-000004](../qas/QAS-000004-security-architecture.md) (the fuller security-architecture context), [QAS-000011](../qas/QAS-000011-multi-tenant-architecture.md)
  (multi-tenant/ownership rules), [QFS-000008](QFS-000008-policy-engine.md) (why there's no generic
  engine above this).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real role list and middleware-based enforcement mechanism. |
