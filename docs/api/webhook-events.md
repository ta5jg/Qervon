# Webhook Event Contract

Qervon integration events use the following JSON envelope:

```json
{
  "event_type": "order.delivered",
  "order_id": "UUID",
  "status": "delivered",
  "timestamp": "RFC3339 UTC timestamp"
}
```

Supported lifecycle event names are `order.created`, `order.assigned`,
`order.in_transit`, `order.delivered`, and `order.cancelled`. Consumers must
treat an event as tenant-scoped and verify the tenant context provided by their
authenticated integration configuration before acting on it.

Customer sessions can manage tenant-scoped subscriptions with
`POST`/`GET /v1/customer/webhooks` and
`DELETE /v1/customer/webhooks/{id}`. Registration accepts only HTTPS endpoints.
The signing secret is returned exactly once at creation; Qervon retains only its
SHA-256 hash. Outgoing delivery, HMAC signing, retry policy and HTTP transport
remain the next integration adapter; subscription management alone does not
assert that events are already delivered.

## Webhook Events

Webhook deliveries use a stable JSON envelope with `id`, `type`, `tenant_id`,
`aggregate_id`, `created_at` and `payload`. The exact serialized UTF-8 body is signed
with the tenant's webhook signing secret and stored as
`sha256=<lowercase-hex-hmac>` for the eventual `X-Qervon-Signature` header.

Supported delivery event:

- `order.delivered`
