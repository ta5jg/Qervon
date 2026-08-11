-- =============================================================================
-- File:           backend/migrations/billing/00000000000001_delivery_invoices.sql
-- Project:        Qervon
-- Description:    Invoice lifecycle storage compatible with the billing domain.
-- =============================================================================

CREATE TABLE billing.delivery_invoices (
    id           uuid PRIMARY KEY,
    order_id     uuid NOT NULL UNIQUE,
    customer_id  uuid NOT NULL,
    amount_minor bigint NOT NULL CHECK (amount_minor > 0),
    currency     char(3) NOT NULL,
    status       text NOT NULL,
    created_at   timestamptz NOT NULL,
    issued_at    timestamptz,
    paid_at      timestamptz,
    CONSTRAINT delivery_invoices_status_check
        CHECK (status IN ('draft', 'issued', 'paid', 'cancelled', 'refunded'))
);

CREATE INDEX delivery_invoices_customer_idx
    ON billing.delivery_invoices (customer_id, created_at DESC);
