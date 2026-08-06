-- =============================================================================
-- File:           backend/migrations/identity/00000000000000_initial.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-05
-- Version:        0.1.0
--
-- Description:
--   Initial governed migration for the Qervon Identity schema.
--
-- Specification:
--   QAS-000001 through QAS-000006, QLS-000001, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS identity;

CREATE TABLE identity.users (
    id           uuid PRIMARY KEY,
    email        text NOT NULL,
    phone        text,
    display_name text NOT NULL,
    role         text NOT NULL DEFAULT 'customer',
    status       text NOT NULL DEFAULT 'active',
    created_at   timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT users_email_unique UNIQUE (email),
    CONSTRAINT users_role_check
        CHECK (role IN ('customer', 'company', 'courier', 'admin', 'super_admin')),
    CONSTRAINT users_status_check
        CHECK (status IN ('active', 'suspended', 'deleted'))
);

CREATE INDEX users_status_idx ON identity.users (status);
