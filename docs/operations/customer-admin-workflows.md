# Customer and Operations Workflows

## Customer order intake

Customer sessions use `/v1/customer/orders`. The service derives the customer
identifier from the signed session; clients cannot submit a customer identifier
in the request body. The created order is bound to the session tenant.

The same endpoint lists only that customer's tenant-bound orders. A customer
cannot read another customer's order list, while live tracking remains limited
to the customer's own assigned orders.

## Operations overview

Operational users can load `/v1/operations/overview`, `/v1/orders`, and
`/v1/couriers`. Every signed request is filtered to its tenant. The overview
reports active and pending orders, in-transit deliveries, courier availability,
and delivered revenue grouped by currency.

## Browser clients

- `/mobile-customer` creates real customer orders, lists current session
  history and follows the first active assigned order without browser storage.
- `/` loads real tenant metrics, order rows, fleet rows, and live locations.

Both screens use the browser session and CSRF-protected write requests.
