-- =============================================================================
-- File:           backend/migrations/billing/00000000000000_initial.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-05
-- Version:        0.1.0
--
-- Description:
--   Initial governed migration for the Qervon Billing schema.
--
-- Specification:
--   QAS-000001 through QAS-000006, QLS-000009, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS billing;

CREATE TABLE billing.invoices (
    id              uuid PRIMARY KEY,
    customer_id     uuid NOT NULL,
    order_id        uuid,
    amount_minor    bigint NOT NULL,
    currency        text NOT NULL,
    status          text NOT NULL DEFAULT 'issued',
    issued_at       timestamptz NOT NULL DEFAULT now(),
    due_at          timestamptz,
    CONSTRAINT invoices_amount_check CHECK (amount_minor >= 0),
    CONSTRAINT invoices_status_check
        CHECK (status IN ('issued', 'paid', 'overdue', 'void'))
);

CREATE TABLE billing.payments (
    id           uuid PRIMARY KEY,
    invoice_id   uuid NOT NULL REFERENCES billing.invoices (id),
    amount_minor bigint NOT NULL,
    currency     text NOT NULL,
    method       text NOT NULL DEFAULT 'cash',
    status       text NOT NULL DEFAULT 'pending',
    paid_at      timestamptz,
    CONSTRAINT payments_amount_check CHECK (amount_minor >= 0),
    CONSTRAINT payments_method_check
        CHECK (method IN ('cash', 'card', 'wallet', 'qr')),
    CONSTRAINT payments_status_check
        CHECK (status IN ('pending', 'captured', 'refunded', 'failed'))
);

CREATE INDEX invoices_customer_idx ON billing.invoices (customer_id, issued_at);
CREATE INDEX payments_invoice_idx ON billing.payments (invoice_id);
