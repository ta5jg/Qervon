-- =============================================================================
-- File:           backend/migrations/dispatch/00000000000002_reoffer_cascade.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-13
-- Version:        0.1.0
--
-- Description:
--   Adds excluded_courier_ids, tracking which couriers a given order has
--   already been offered to and who rejected/let the offer expire, so the
--   automatic re-offer cascade (DispatchService::reoffer_from_candidates)
--   never offers the same job to the same courier twice.
--
-- Specification:
--   QAS-000002, QAS-000003, QLS-000003, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

ALTER TABLE dispatch.assignments
    ADD COLUMN IF NOT EXISTS excluded_courier_ids uuid[] NOT NULL DEFAULT '{}';
