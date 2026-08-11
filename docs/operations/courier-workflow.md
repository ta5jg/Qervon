# Courier Work Workflow

## Identity and access

A courier session is bound to a courier profile with the same identifier. The
profile and every order visible to the courier must belong to the token tenant.
The courier API has no arbitrary courier identifier in its URL, so a signed-in
courier cannot switch to another courier profile by changing a request path.

## Operational flow

1. Dispatch assigns an available, tenant-owned courier to a pending order.
2. The courier sees that assigned active order at `GET /v1/courier/orders`.
3. `POST /v1/courier/orders/{id}/pickup` moves the order to `in_transit`.
4. `POST /v1/courier/orders/{id}/deliver` completes only an in-transit order.
5. Completion returns the courier to `available`.

The courier may mark itself offline only while available. It can return online
through `POST /v1/courier/me/status`. GPS is published through
`POST /v1/courier/me/location`; the server obtains the courier identifier from
the authenticated session.

## Mobile terminal

`/mobile-courier` loads the authenticated courier profile and active work from
these endpoints. Its status toggle, pickup, delivery and GPS publishing all
use the browser session and CSRF protection; it does not persist a bearer token
or courier identifier in browser storage.
