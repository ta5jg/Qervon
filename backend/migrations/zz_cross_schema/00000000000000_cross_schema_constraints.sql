-- =============================================================================
-- File:           backend/migrations/zz_cross_schema/00000000000000_cross_schema_constraints.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-05
-- Version:        0.1.0
--
-- Description:
--   Adds cross-schema foreign keys that must be applied after every module
--   schema exists. This directory sorts after all module directories so the
--   referenced tables are guaranteed to exist.
--
-- Specification:
--   QAS-000001 through QAS-000006, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

ALTER TABLE billing.invoices
    ADD CONSTRAINT invoices_order_fk
    FOREIGN KEY (order_id) REFERENCES orders.orders (id);
