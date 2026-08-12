-- =============================================================================
-- File:           backend/migrations/identity/00000000000003_otp_challenges.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-12
-- Version:        0.1.0
--
-- Description:
--   Stores one-time-password (OTP) challenges for phone-based login, and a
--   uniqueness guarantee on identity.users.phone so a phone number resolves
--   to at most one account. The tenant_id foreign key is added later in
--   zz_cross_schema once the tenancy schema exists.
--
-- Specification:
--   QAS-000002, QAS-000004, QLS-000001, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE UNIQUE INDEX IF NOT EXISTS users_phone_unique_idx
    ON identity.users (phone)
    WHERE phone IS NOT NULL;

CREATE TABLE identity.otp_challenges (
    id           uuid PRIMARY KEY,
    tenant_id    uuid NOT NULL,
    phone        text NOT NULL,
    code_hash    text NOT NULL,
    attempts     smallint NOT NULL DEFAULT 0,
    created_at   timestamptz NOT NULL DEFAULT now(),
    expires_at   timestamptz NOT NULL,
    consumed_at  timestamptz,
    CONSTRAINT otp_challenges_attempts_check CHECK (attempts >= 0)
);

CREATE INDEX otp_challenges_lookup_idx
    ON identity.otp_challenges (tenant_id, phone, created_at DESC);
