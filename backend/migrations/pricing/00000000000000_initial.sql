-- =============================================================================
-- File:           backend/migrations/pricing/00000000000000_initial.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-12
-- Version:        0.1.0
--
-- Description:
--   Initial governed migration for the Qervon Pricing schema: one
--   distance-based delivery pricing configuration per tenant. A tenant with
--   no row here still gets a real fare via the application-layer default
--   (see qervon_domain::DeliveryPricing::default_for_tenant) — this table
--   only holds tenant-specific overrides.
--
-- Specification:
--   QAS-000005, QAS-000006, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS pricing;

CREATE TABLE pricing.delivery_pricing (
    tenant_id             uuid PRIMARY KEY,
    base_fare_minor       bigint NOT NULL,
    per_km_rate_minor     bigint NOT NULL,
    minimum_fare_minor    bigint NOT NULL,
    currency              text NOT NULL,
    updated_at            timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT delivery_pricing_base_fare_check CHECK (base_fare_minor >= 0),
    CONSTRAINT delivery_pricing_per_km_rate_check CHECK (per_km_rate_minor >= 0),
    CONSTRAINT delivery_pricing_minimum_fare_check CHECK (minimum_fare_minor >= 0)
);
