-- =============================================================================
-- File:           backend/migrations/zz_cross_schema/00000000000003_vehicle_tenants.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-12
-- Version:        0.1.0
--
-- Description:
--   Adds tenant ownership binding for fleet vehicles, mirroring the existing
--   courier_tenants/order_tenants pattern. Required before exposing any
--   /v1/fleet HTTP routes, since vehicles previously had no tenant boundary.
--
-- Specification:
--   QAS-000006, QAS-000011, QLS-000006, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE TABLE tenancy.vehicle_tenants (
    vehicle_id uuid PRIMARY KEY REFERENCES fleet.vehicles (id),
    tenant_id uuid NOT NULL REFERENCES tenancy.tenants (id)
);
CREATE INDEX vehicle_tenants_tenant_idx ON tenancy.vehicle_tenants (tenant_id);
