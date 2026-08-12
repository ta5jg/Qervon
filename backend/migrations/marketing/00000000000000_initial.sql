-- =============================================================================
-- File:           backend/migrations/marketing/00000000000000_initial.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-12
-- Version:        0.1.0
--
-- Description:
--   Initial governed migration for the Qervon Marketing schema: tenant-scoped
--   promo coupons.
--
-- Specification:
--   QAS-000005, QAS-000006, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS marketing;

CREATE TABLE marketing.coupons (
    id                    uuid PRIMARY KEY,
    tenant_id             uuid NOT NULL,
    code                  text NOT NULL,
    discount_percent      double precision NOT NULL,
    max_discount_minor    bigint NOT NULL,
    valid_until           timestamptz NOT NULL,
    usage_limit           integer NOT NULL,
    used_count            integer NOT NULL DEFAULT 0,
    is_active             boolean NOT NULL DEFAULT true,
    created_at            timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT coupons_tenant_code_unique UNIQUE (tenant_id, code),
    CONSTRAINT coupons_discount_percent_check CHECK (discount_percent BETWEEN 0 AND 100),
    CONSTRAINT coupons_max_discount_check CHECK (max_discount_minor >= 0),
    CONSTRAINT coupons_usage_limit_check CHECK (usage_limit > 0),
    CONSTRAINT coupons_used_count_check CHECK (used_count >= 0)
);
