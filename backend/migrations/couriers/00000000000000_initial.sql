-- =============================================================================
-- File:           backend/migrations/couriers/00000000000000_initial.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-05
-- Version:        0.1.0
--
-- Description:
--   Initial governed migration for the Qervon Couriers schema.
--
-- Specification:
--   QAS-000002, QAS-000005, QAS-000006, QLS-000004.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS couriers;

CREATE TABLE couriers.couriers (
    id             uuid PRIMARY KEY,
    name           text NOT NULL,
    vehicle        text NOT NULL,
    status         text NOT NULL DEFAULT 'available',
    current_lat    double precision,
    current_lon    double precision,
    registered_at  timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT couriers_status_check
        CHECK (status IN ('available', 'busy', 'offline')),
    CONSTRAINT couriers_vehicle_check
        CHECK (vehicle IN ('bicycle', 'motorcycle', 'car'))
);

CREATE INDEX couriers_status_idx ON couriers.couriers (status);
