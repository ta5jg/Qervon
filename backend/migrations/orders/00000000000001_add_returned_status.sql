-- =============================================================================
-- File:           backend/migrations/orders/00000000000001_add_returned_status.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-12
-- Version:        0.1.0
--
-- Description:
--   Adds the `returned` order status (a package returned mid-route or after
--   delivery) and the timestamp at which the return was recorded.
--
-- Specification:
--   QAS-000002, QAS-000006, QLS-000002, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

ALTER TABLE orders.orders
    ADD COLUMN IF NOT EXISTS returned_at timestamptz;

ALTER TABLE orders.orders
    DROP CONSTRAINT orders_status_check;

ALTER TABLE orders.orders
    ADD CONSTRAINT orders_status_check
    CHECK (status IN ('pending', 'courier_assigned', 'in_transit', 'delivered', 'cancelled', 'returned'));
