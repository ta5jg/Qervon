-- =============================================================================
-- File:           backend/migrations/tracking/00000000000003_route_breadcrumbs_tenant_scoping.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-13
-- Version:        0.1.0
--
-- Description:
--   Adds tenant ownership to GPS route breadcrumbs so a tenant can only
--   ever see the breadcrumb trail of couriers bound to it (see
--   BACKEND_BACKLOG.md "Domain genişlemesi" follow-up). The table was added
--   in 00000000000002_route_history.sql without any writer yet, so this is
--   a plain non-nullable column addition with no backfill needed.
--
-- Specification:
--   QAS-000002, QAS-000003, QAS-000006, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

ALTER TABLE tracking.route_breadcrumbs ADD COLUMN tenant_id uuid NOT NULL;

CREATE INDEX IF NOT EXISTS idx_route_breadcrumbs_tenant
    ON tracking.route_breadcrumbs (tenant_id, courier_id, recorded_at DESC);
