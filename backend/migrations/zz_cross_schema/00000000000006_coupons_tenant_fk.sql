-- =============================================================================
-- File:           backend/migrations/zz_cross_schema/00000000000006_coupons_tenant_fk.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-12
-- Version:        0.1.0
--
-- Description:
--   Adds the tenant relationship for marketing coupons after both marketing
--   and tenancy schemas exist.
--
-- Specification:
--   QAS-000006, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

ALTER TABLE marketing.coupons
    ADD CONSTRAINT coupons_tenant_fk
    FOREIGN KEY (tenant_id) REFERENCES tenancy.tenants (id);
