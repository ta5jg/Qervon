-- =============================================================================
-- File:           backend/migrations/dispatch/00000000000001_offer_expiry.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-12
-- Version:        0.1.0
--
-- Description:
--   Adds an `offered` assignment status plus offer/response timestamps so a
--   job can be offered to a courier and explicitly accepted or rejected,
--   instead of being assigned instantly and unconditionally.
--
-- Specification:
--   QAS-000002, QAS-000003, QLS-000003, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

ALTER TABLE dispatch.assignments
    ADD COLUMN IF NOT EXISTS offered_at timestamptz,
    ADD COLUMN IF NOT EXISTS responded_at timestamptz;

-- Backfill existing rows (all currently 'assigned' or 'completed') so
-- offered_at is never null for historical data.
UPDATE dispatch.assignments SET offered_at = assigned_at WHERE offered_at IS NULL;

ALTER TABLE dispatch.assignments
    ALTER COLUMN offered_at SET NOT NULL;

ALTER TABLE dispatch.assignments
    DROP CONSTRAINT assignments_status_check;

ALTER TABLE dispatch.assignments
    ADD CONSTRAINT assignments_status_check
    CHECK (status IN ('offered', 'assigned', 'completed', 'cancelled'));
