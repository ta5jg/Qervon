CREATE SCHEMA IF NOT EXISTS integrations;
CREATE TABLE integrations.webhooks (
    id uuid PRIMARY KEY,
    tenant_id uuid NOT NULL REFERENCES tenancy.tenants(id),
    endpoint_url text NOT NULL CHECK (endpoint_url LIKE 'https://%'),
    event_types text[] NOT NULL CHECK (cardinality(event_types) > 0),
    secret_hash char(64) NOT NULL,
    enabled boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL
);
CREATE INDEX integrations_webhooks_tenant_idx ON integrations.webhooks (tenant_id, created_at DESC);
