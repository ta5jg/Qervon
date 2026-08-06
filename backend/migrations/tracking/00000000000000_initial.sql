-- =============================================================================
-- File:           backend/migrations/tracking/00000000000000_initial.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-05
-- Version:        0.1.0
--
-- Description:
--   Initial governed migration for the Qervon Tracking schema.
--
-- Specification:
--   QAS-000001 through QAS-000006, QLS-000007, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS tracking;

CREATE TABLE tracking.events (
    id          uuid PRIMARY KEY,
    order_id    uuid NOT NULL REFERENCES orders.orders (id),
    courier_id  uuid REFERENCES couriers.couriers (id),
    event_type  text NOT NULL,
    occurred_at timestamptz NOT NULL DEFAULT now(),
    payload     jsonb NOT NULL DEFAULT '{}'::jsonb,
    CONSTRAINT events_event_type_check
        CHECK (event_type IN ('order_created', 'courier_assigned', 'in_transit', 'delivered', 'cancelled'))
);

CREATE INDEX events_order_idx ON tracking.events (order_id, occurred_at);
CREATE INDEX events_courier_idx ON tracking.events (courier_id, occurred_at);
