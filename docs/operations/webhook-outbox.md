# Webhook Outbox

`integrations.event_outbox` is the durable, tenant-scoped queue created in the
same PostgreSQL transaction as a completed delivery. The worker claims eligible
rows with `FOR UPDATE SKIP LOCKED`, decrypts the tenant webhook secret, builds a
stable JSON envelope and persists its exact UTF-8 bytes plus one signed row per matching subscription in
`integrations.webhook_delivery_outbox`.

The source event is marked processed only after its complete fan-out is stored.
Failure advances `attempts` with a 30-second exponential backoff (capped at one
hour); the event becomes a dead letter after `QERVON_WEBHOOK_MAX_ATTEMPTS`.
Each endpoint-delivery row has its own claim, retry and dead-letter fields. The
worker sends its stored bytes through HTTPS with `X-Qervon-Event`,
`X-Qervon-Delivery`, and `X-Qervon-Signature` headers. Only a 2xx response marks
that endpoint delivery complete; failures retry with the same exponential
backoff and eventually become a delivery-level dead letter.

No HTTP request originates in the API process. Production egress is enabled
only by the worker and only for HTTPS hosts listed in
`QERVON_WEBHOOK_ALLOWED_HOSTS`; an empty allow-list fails closed.
