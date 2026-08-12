-- =============================================================================
-- File:           backend/migrations/tracking/00000000000002_fraud_signal.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-12
-- Version:        0.1.0
--
-- Description:
--   Adds the AI Fraud Guard's flag-and-accept signal to recorded location
--   samples: a point is never rejected, but implausible-speed samples are
--   annotated so operators can see them.
--
-- Specification:
--   QAS-000002, QAS-000003, QAS-000006, QLS-000007, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

ALTER TABLE tracking.location_points
    ADD COLUMN IF NOT EXISTS fraud_flagged boolean NOT NULL DEFAULT false,
    ADD COLUMN IF NOT EXISTS fraud_risk_score double precision NOT NULL DEFAULT 0
        CHECK (fraud_risk_score BETWEEN 0 AND 1);

CREATE INDEX IF NOT EXISTS tracking_location_points_fraud_idx
    ON tracking.location_points (courier_id)
    WHERE fraud_flagged;
