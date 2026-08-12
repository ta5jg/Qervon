-- =============================================================================
-- File:           backend/migrations/orders/00000000000003_delivery_note_contact_phone.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-12
-- Version:        0.1.0
--
-- Description:
--   Adds optional, set-once-at-creation delivery instructions and a
--   dropoff contact phone number to orders.
--
-- Specification:
--   QAS-000002, QLS-000002, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

ALTER TABLE orders.orders
    ADD COLUMN IF NOT EXISTS delivery_note text,
    ADD COLUMN IF NOT EXISTS contact_phone text;
