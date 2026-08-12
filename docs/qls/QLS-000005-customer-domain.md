<!-- =============================================================================
File:           docs/qls/QLS-000005-customer-domain.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The CustomerProfile entity: address book, loyalty points, and
  registration.

Specification:
  QAS-000002, QAS-000011, QLS-000014.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000005 — Customer Domain

**Status: Implemented.**

## Registration

`POST /v1/auth/register` creates a `customer`-role user plus a
`CustomerProfile` together, and (if a `tenant_slug` was supplied) a
tenant membership. Public registration can never create any role other
than `customer` — every other role (courier, admin, operator, ...) is
provisioned by an existing tenant admin (see QAS-000011). Registration
returns no tokens — the client must follow up with a real password login,
matching the actual backend contract exactly (both mobile apps and the
web `login.html` do this).

## Profile fields

`id`, `user_id`, `company_name: Option<String>`, `tax_id:
Option<String>`, `addresses: Vec<SavedAddress>`, `loyalty_points: u64`,
`created_at`.

## Address book

`SavedAddress { id, label, location, full_address, is_default }`,
managed via `POST`/`DELETE /v1/customer/profile/addresses[/{id}]`. Both
native apps provide a map-based picker (MapKit on iOS, osmdroid on
Android) plus device-local reverse geocoding to fill in `full_address` —
no backend geocoding call exists (see QAS-000007).

## Loyalty points

A real integer field with a real service method
(`CustomerService::add_loyalty_points`, domain-validated, tested), but
**no HTTP endpoint or automatic trigger calls it** — there is no "award
points on delivery" hook, no admin endpoint to grant points manually. The
capability to award points exists and is exercised by unit/integration
tests, but nothing in the live request-handling code path invokes it
today, so every real account's `loyalty_points` stays at its initial
value. Both customer apps display the field as-is. Wiring an actual
trigger (most naturally: award points on order delivery, in the same
place the courier wallet gets credited) is a small, well-scoped follow-up
— the hard part (the domain method) already exists.

## What is not implemented

- No loyalty-point earning/redemption rules.
- No "favorites" concept beyond the address book (a favorite courier or
  favorite past order does not exist as a distinct feature — see
  QLS-000014).

## References

- [QAS-000002](../qas/QAS-000002-domain-model.md) (domain model), [QAS-000011](../qas/QAS-000011-multi-tenant-architecture.md) (tenant/provisioning rules),
  [QLS-000014](QLS-000014-customer-experience.md) (customer experience — ratings/support/notifications this
  profile connects to).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real profile/address-book behavior and the honest loyalty-points gap. |
