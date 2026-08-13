-- =============================================================================
-- File:           backend/migrations/warehouse/00000000000001_tenant_scoping.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-13
-- Version:        0.1.0
--
-- Description:
--   Adds tenant ownership to warehouse hubs so hub CRUD can be scoped per
--   tenant the same way every other operational resource in the platform
--   is (see BACKEND_BACKLOG.md "Domain genişlemesi" follow-up). The table
--   was added in 00000000000000_initial.sql without any writer yet, so this
--   is a plain non-nullable column addition with no backfill needed.
--
-- Specification:
--   QAS-000002, QAS-000006, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

ALTER TABLE warehouse.hubs ADD COLUMN tenant_id uuid NOT NULL;

CREATE INDEX IF NOT EXISTS warehouse_hubs_tenant_idx ON warehouse.hubs (tenant_id);
