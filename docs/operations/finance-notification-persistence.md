# Finance and Notification Persistence

The production PostgreSQL composition now persists delivery invoices and
customer notifications instead of retaining them in process memory.

- `billing.delivery_invoices` stores the domain invoice lifecycle (`draft`,
  `issued`, `paid`, `cancelled`, `refunded`) with one invoice per order.
- `notifications.notifications` stores queued, sent, failed, and read customer
  notifications, ordered by creation time for customer retrieval.

Both tables are created by the normal governed migration runner. Apply the
migrations before deploying a release that uses `QERVON_STORAGE=postgres`.
Existing legacy `billing.invoices` records are intentionally not rewritten by
this additive change; no production data is modified without an explicit,
verified migration plan.
