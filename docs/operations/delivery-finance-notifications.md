# Delivery Finance and Notifications

When a courier completes an in-transit order, Qervon performs two idempotent
follow-up operations:

1. It creates and issues one invoice for the delivered order if no invoice
   already exists.
2. It queues a push notification for the customer confirming delivery.

Customers can retrieve an invoice only for their own tenant-bound order at
`GET /v1/customer/orders/{id}/invoice`, and can retrieve only their own
notifications at `GET /v1/customer/notifications`.

Delivery itself remains the source of truth. Financial records and
notifications are created immediately after the delivery transition and are
protected by the order's tenant and customer ownership checks.

Proof-of-delivery media persistence and external notification delivery adapters
are separate integrations. The courier delivery endpoint now requires recipient
identity plus at least one proof signal (QR verification, signature, or photo)
and makes the resulting POD record available only to the owning customer. The
current notification record represents the application lifecycle in the
configured repository, not a claim that an external push provider has accepted
delivery.
