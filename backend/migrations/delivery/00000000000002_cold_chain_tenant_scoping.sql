-- =============================================================================
-- File:           backend/migrations/delivery/00000000000002_cold_chain_tenant_scoping.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-13
-- Version:        0.1.0
--
-- Description:
--   Adds tenant ownership to cold-chain telemetry readings so a tenant can
--   only ever see its own sensor data (see BACKEND_BACKLOG.md "Domain
--   genişlemesi" follow-up). The table was added in
--   00000000000001_cold_chain.sql without any writer yet, so this is a
--   plain non-nullable column addition with no backfill needed.
--
-- Specification:
--   QAS-000002, QAS-000006, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

ALTER TABLE delivery.cold_chain_telemetry ADD COLUMN tenant_id uuid NOT NULL;

CREATE INDEX IF NOT EXISTS idx_cold_chain_tenant
    ON delivery.cold_chain_telemetry (tenant_id, recorded_at DESC);
