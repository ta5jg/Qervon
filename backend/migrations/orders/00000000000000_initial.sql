-- =============================================================================
-- File:           backend/migrations/orders/00000000000000_initial.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-05
-- Version:        0.1.0
--
-- Description:
--   Initial governed migration for the Qervon Orders schema.
--
-- Specification:
--   QAS-000002, QAS-000005, QAS-000006, QLS-000002.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS orders;

CREATE TABLE orders.orders (
    id                 uuid PRIMARY KEY,
    customer_id        uuid NOT NULL,
    pickup_lat         double precision NOT NULL,
    pickup_lon         double precision NOT NULL,
    pickup_label       text,
    dropoff_lat        double precision NOT NULL,
    dropoff_lon        double precision NOT NULL,
    dropoff_label      text,
    status             text NOT NULL DEFAULT 'pending',
    fare_amount_minor  bigint NOT NULL,
    fare_currency      char(3) NOT NULL,
    assigned_courier_id uuid,
    created_at         timestamptz NOT NULL DEFAULT now(),
    delivered_at       timestamptz,
    CONSTRAINT orders_status_check
        CHECK (status IN ('pending', 'courier_assigned', 'in_transit', 'delivered', 'cancelled')),
    CONSTRAINT orders_fare_check
        CHECK (fare_amount_minor >= 0)
);

CREATE INDEX orders_status_idx ON orders.orders (status);
CREATE INDEX orders_customer_idx ON orders.orders (customer_id);
