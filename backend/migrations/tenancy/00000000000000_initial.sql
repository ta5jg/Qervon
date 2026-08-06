-- =============================================================================
-- File:           backend/migrations/tenancy/00000000000000_initial.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-05
-- Version:        0.1.0
--
-- Description:
--   Initial governed migration for the Qervon Tenancy schema.
--
-- Specification:
--   QAS-000001 through QAS-000006, QLS-000001, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS tenancy;

CREATE TABLE tenancy.tenants (
    id         uuid PRIMARY KEY,
    name       text NOT NULL,
    slug       text NOT NULL,
    status     text NOT NULL DEFAULT 'active',
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT tenants_slug_unique UNIQUE (slug),
    CONSTRAINT tenants_status_check
        CHECK (status IN ('active', 'suspended'))
);

CREATE TABLE tenancy.tenant_members (
    tenant_id uuid NOT NULL REFERENCES tenancy.tenants (id),
    user_id   uuid NOT NULL REFERENCES identity.users (id),
    role      text NOT NULL DEFAULT 'member',
    joined_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (tenant_id, user_id),
    CONSTRAINT tenant_members_role_check
        CHECK (role IN ('owner', 'admin', 'operator', 'member'))
);

CREATE INDEX tenant_members_user_idx ON tenancy.tenant_members (user_id);
