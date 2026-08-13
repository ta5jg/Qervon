CREATE TABLE IF NOT EXISTS billing.tax_invoice_drafts (
    id uuid PRIMARY KEY,
    invoice_number text NOT NULL UNIQUE,
    order_id uuid NOT NULL,
    customer_id uuid NOT NULL,
    net_amount_minor bigint NOT NULL,
    vat_amount_minor bigint NOT NULL,
    total_amount_minor bigint NOT NULL,
    vat_rate_percent double precision NOT NULL,
    currency text NOT NULL,
    issued_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS billing.currency_rate_snapshots (
    id uuid PRIMARY KEY,
    source_currency text NOT NULL,
    target_currency text NOT NULL,
    rate double precision NOT NULL,
    captured_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_currency_rate_pair_time
    ON billing.currency_rate_snapshots (source_currency, target_currency, captured_at DESC);
