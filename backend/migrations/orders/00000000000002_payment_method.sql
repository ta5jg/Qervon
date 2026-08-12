-- =============================================================================
-- File:           backend/migrations/orders/00000000000002_payment_method.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-12
-- Version:        0.1.0
--
-- Description:
--   Adds the chosen payment method and a courier-side collection flag to
--   orders. There is no real payment gateway behind this: card/QR/wallet
--   only record the selected method, they do not process a transaction.
--
-- Specification:
--   QAS-000002, QLS-000002, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

ALTER TABLE orders.orders
    ADD COLUMN IF NOT EXISTS payment_method text,
    ADD COLUMN IF NOT EXISTS payment_collected boolean NOT NULL DEFAULT false;

ALTER TABLE orders.orders
    ADD CONSTRAINT orders_payment_method_check
    CHECK (payment_method IS NULL OR payment_method IN ('cash', 'card', 'qr', 'wallet'));
