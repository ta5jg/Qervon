-- =============================================================================
-- File:           backend/migrations/service/00000000000001_tenant_scoping.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-13
-- Version:        0.1.0
--
-- Description:
--   Adds tenant ownership to field-service appointments so a tenant can
--   only ever see its own appointments (see BACKEND_BACKLOG.md "Domain
--   genişlemesi" follow-up). The table was added in
--   00000000000000_field_service.sql without any writer yet, so this is a
--   plain non-nullable column addition with no backfill needed.
--
-- Specification:
--   QAS-000002, QAS-000006, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

ALTER TABLE service.field_service_appointments ADD COLUMN tenant_id uuid NOT NULL;

CREATE INDEX IF NOT EXISTS idx_field_service_tenant
    ON service.field_service_appointments (tenant_id, appointment_date DESC);
