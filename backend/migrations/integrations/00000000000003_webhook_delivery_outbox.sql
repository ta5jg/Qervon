CREATE TABLE IF NOT EXISTS integrations.webhook_delivery_outbox (
    id uuid PRIMARY KEY,
    event_outbox_id uuid NOT NULL REFERENCES integrations.event_outbox(id) ON DELETE CASCADE,
    webhook_id uuid NOT NULL REFERENCES integrations.webhooks(id) ON DELETE CASCADE,
    tenant_id uuid NOT NULL REFERENCES tenancy.tenants(id),
    endpoint_url text NOT NULL CHECK (endpoint_url LIKE 'https://%'),
    event_type text NOT NULL,
    aggregate_id uuid NOT NULL,
    payload jsonb NOT NULL,
    body bytea NOT NULL,
    signature text NOT NULL,
    attempts integer NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    available_at timestamptz NOT NULL DEFAULT now(),
    claimed_at timestamptz,
    claim_token uuid,
    delivered_at timestamptz,
    dead_lettered_at timestamptz,
    last_error text,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (event_outbox_id, webhook_id)
);

CREATE INDEX IF NOT EXISTS webhook_delivery_outbox_claimable_idx
    ON integrations.webhook_delivery_outbox (available_at, created_at)
    WHERE delivered_at IS NULL AND dead_lettered_at IS NULL;
