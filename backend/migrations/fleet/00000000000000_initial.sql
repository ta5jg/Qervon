-- =============================================================================
-- File:           backend/migrations/fleet/00000000000000_initial.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-05
-- Version:        0.1.0
--
-- Description:
--   Initial governed migration for the Qervon Fleet schema.
--
-- Specification:
--   QAS-000001 through QAS-000006, QLS-000006, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS fleet;

CREATE TABLE fleet.vehicles (
    id          uuid PRIMARY KEY,
    courier_id  uuid REFERENCES couriers.couriers (id),
    kind        text NOT NULL,
    make        text,
    model       text,
    plate       text,
    status      text NOT NULL DEFAULT 'operational',
    created_at  timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT vehicles_kind_check
        CHECK (kind IN ('bicycle', 'motorcycle', 'car')),
    CONSTRAINT vehicles_status_check
        CHECK (status IN ('operational', 'maintenance', 'retired'))
);

CREATE INDEX vehicles_courier_idx ON fleet.vehicles (courier_id);
CREATE INDEX vehicles_status_idx ON fleet.vehicles (status);
