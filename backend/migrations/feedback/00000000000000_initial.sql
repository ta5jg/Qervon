-- =============================================================================
-- File:           backend/migrations/feedback/00000000000000_initial.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-12
-- Version:        0.1.0
--
-- Description:
--   Initial governed migration for the Qervon Feedback schema: customer
--   ratings for delivered orders and customer support tickets.
--
-- Specification:
--   QAS-000002, QAS-000004, QAS-000006, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS feedback;

CREATE TABLE feedback.customer_ratings (
    id            uuid PRIMARY KEY,
    order_id      uuid NOT NULL UNIQUE REFERENCES orders.orders (id),
    customer_id   uuid NOT NULL,
    courier_id    uuid NOT NULL REFERENCES couriers.couriers (id),
    rating_stars  smallint NOT NULL,
    comment       text,
    photo_url     text,
    created_at    timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT customer_ratings_stars_check CHECK (rating_stars BETWEEN 1 AND 5)
);

CREATE INDEX customer_ratings_courier_idx ON feedback.customer_ratings (courier_id);

CREATE TABLE feedback.support_tickets (
    id           uuid PRIMARY KEY,
    tenant_id    uuid NOT NULL,
    customer_id  uuid NOT NULL,
    order_id     uuid REFERENCES orders.orders (id),
    subject      text NOT NULL,
    message      text NOT NULL,
    status       text NOT NULL DEFAULT 'open',
    created_at   timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT support_tickets_status_check
        CHECK (status IN ('open', 'in_progress', 'resolved', 'closed'))
);

CREATE INDEX support_tickets_customer_idx ON feedback.support_tickets (customer_id, created_at DESC);
CREATE INDEX support_tickets_tenant_idx ON feedback.support_tickets (tenant_id, status);
