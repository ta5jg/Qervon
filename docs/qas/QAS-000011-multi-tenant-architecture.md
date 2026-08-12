<!-- =============================================================================
File:           docs/qas/QAS-000011-multi-tenant-architecture.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  How tenant isolation is enforced across auth, the API, and the data
  layer.

Specification:
  QAS-000004, QAS-000006.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000011 — Multi-Tenant Architecture

**Status: Implemented.**

## Model

Every tenant is a row in `tenancy.tenants` with a unique `slug` (used in
login) and `name`. Every user has zero or more `TenantMembership` rows
(user_id, tenant_id, role) — a user is not implicitly a member of every
tenant just because an account exists; `TenantMemberRole` (Owner, Admin,
Member, ...) is distinct from the user's global `role` (which governs
what kind of account it is — customer, courier, admin, super_admin).

Core resources (orders, couriers) are bound to exactly one tenant at
creation and never move between tenants. A few originally-tenant-agnostic
resources (vehicles) gained an explicit `vehicle_tenants` binding table
during Faz-1 specifically to close a gap where fleet assets weren't yet
tenant-scoped.

## Enforcement layers

1. **Login-time:** authenticating requires a valid `tenant_slug` +
   email + password combination *and* an existing membership row for
   that user in that tenant — a correct password for the wrong tenant
   slug fails, even for a user who is a real member of a different
   tenant.
2. **Token-time:** every access token embeds `tenant_id` (see
   QAS-000004); every subsequent request is implicitly scoped to that
   tenant for the life of the token.
3. **Query-time:** every repository method that reads/writes a
   tenant-scoped entity takes/filters by `tenant_id` explicitly — there
   is no PostgreSQL Row-Level Security policy doing this implicitly;
   isolation is enforced in application code, verified by integration
   tests that assert cross-tenant access is rejected (see
   `backend/apps/api-gateway/tests/api_flow.rs`, e.g. the "admin overview
   is tenant scoped" test).
4. **Resource-ownership checks:** beyond tenant scoping, a customer can
   only act on their *own* orders (`require_customer_order`), and a
   courier only on orders assigned to *them* (`require_courier_order`) —
   two customers in the same tenant cannot see each other's orders.

## Provisioning

- The very first tenant + its owner are created once, out-of-band, via
  either the `/setup` screen (local/dev) or a one-time VPS bootstrap
  command (production) — never through the public registration API,
  which can only ever create a `customer` account (see ADR-000008's
  sibling discussion in `auth_register`).
- Every subsequent tenant is provisioned by an existing platform
  super-admin (`POST /v1/tenants/provision`), which creates the tenant
  and its first admin together in one call.

## What is not implemented

- No per-tenant resource quotas/rate-limit tiers — the rate limiter
  (QAS-000004) is IP-keyed, not tenant-keyed.
- No tenant-level feature flags or plan tiers.
- No tenant data export/deletion self-service tooling (GDPR-style).

## References

- [QAS-000004](QAS-000004-security-architecture.md) (auth this relies on), [QAS-000006](QAS-000006-database-persistence-standard.md) (the data-layer
  implementation), [docs/operations/deployment-runbook.md](../operations/deployment-runbook.md)
  (tenant bootstrap procedure).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real four-layer enforcement model and provisioning flow. |
