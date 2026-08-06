-- =============================================================================
-- File:           backend/migrations/dispatch/00000000000000_initial.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-05
-- Version:        0.1.0
--
-- Description:
--   Initial governed migration for the Qervon Dispatch schema.
--
-- Specification:
--   QAS-000002, QAS-000005, QAS-000006, QLS-000003.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS dispatch;

CREATE TABLE dispatch.assignments (
    id           uuid PRIMARY KEY,
    order_id     uuid NOT NULL,
    courier_id   uuid NOT NULL,
    status       text NOT NULL DEFAULT 'assigned',
    assigned_at  timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT assignments_status_check
        CHECK (status IN ('assigned', 'completed', 'cancelled')),
    CONSTRAINT assignments_order_unique
        UNIQUE (order_id)
);

CREATE INDEX assignments_courier_idx ON dispatch.assignments (courier_id);
