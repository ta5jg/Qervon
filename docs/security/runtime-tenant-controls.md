# Runtime Tenant Controls

This document defines the authorization boundary currently enforced by the
Qervon API gateway for courier, order, dispatch, and live-location traffic.

## Ownership boundary

- A signed user token carries one tenant identifier and one role.
- New couriers and orders created through a signed operational session are
  bound to that tenant.
- Signed requests may list, read, change, track, or assign only resources
  bound to the same tenant.
- A customer may only track an order whose `customer_id` matches the token
  subject.
- Live locations and WebSocket events are filtered by tenant before delivery.

## Dispatch boundary

Interactive automatic dispatch first filters available couriers by tenant
ownership, then ranks only that filtered set. A courier belonging to another
tenant is never an automatic candidate, even when it is geographically closer.

Trusted service-token calls are an explicit system integration path. They do
not represent an end-user tenant session and must be issued only to controlled
backend jobs. New integrations should prefer a signed tenant context whenever
they act on behalf of a tenant.

## Browser session boundary

Browser login creates short-lived HttpOnly access and refresh cookies plus a
separate CSRF cookie. Cookie-authenticated write requests must echo the CSRF
value in `X-CSRF-Token`. Refresh tokens rotate on use; logout revokes the active
refresh session and expires all browser cookies.

The access and refresh cookies are marked `Secure`; local browser testing
therefore requires HTTPS rather than weakening the production cookie policy.

## Verification

The API integration suite covers cross-tenant live tracking, manual assignment,
automatic assignment, CSRF rejection and acceptance, refresh rotation, refresh
reuse rejection, and logout cookie expiry.
