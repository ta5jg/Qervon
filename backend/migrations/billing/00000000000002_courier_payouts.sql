-- =============================================================================
-- File:           backend/migrations/billing/00000000000002_courier_payouts.sql
-- Description:    Durable courier payout records.
-- =============================================================================

CREATE TABLE billing.courier_payouts (
    id                       uuid PRIMARY KEY,
    courier_id               uuid NOT NULL REFERENCES couriers.couriers (id),
    period_start             timestamptz NOT NULL,
    period_end               timestamptz NOT NULL,
    gross_amount_minor       bigint NOT NULL CHECK (gross_amount_minor >= 0),
    commission_amount_minor  bigint NOT NULL CHECK (commission_amount_minor >= 0),
    net_amount_minor         bigint NOT NULL CHECK (net_amount_minor >= 0),
    currency                 char(3) NOT NULL,
    status                   text NOT NULL,
    created_at               timestamptz NOT NULL,
    CONSTRAINT courier_payouts_period_check CHECK (period_end > period_start),
    CONSTRAINT courier_payouts_amount_check
        CHECK (gross_amount_minor - commission_amount_minor = net_amount_minor),
    CONSTRAINT courier_payouts_status_check CHECK (status IN ('pending', 'approved', 'paid'))
);

CREATE INDEX courier_payouts_courier_period_idx
    ON billing.courier_payouts (courier_id, period_start DESC);
