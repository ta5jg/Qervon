-- Safe additive upgrade: the initial migration may already be recorded in production.
ALTER TABLE integrations.webhooks
    ADD COLUMN IF NOT EXISTS encrypted_secret bytea;

ALTER TABLE integrations.event_outbox
    ADD COLUMN IF NOT EXISTS claimed_at timestamptz,
    ADD COLUMN IF NOT EXISTS claim_token uuid,
    ADD COLUMN IF NOT EXISTS dead_lettered_at timestamptz;

CREATE INDEX IF NOT EXISTS event_outbox_claimable_idx
    ON integrations.event_outbox (available_at, created_at)
    WHERE delivered_at IS NULL AND dead_lettered_at IS NULL;
