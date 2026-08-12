-- =============================================================================
-- File:           backend/migrations/fleet/00000000000001_vehicle_repository.sql
-- Description:    Adds the fields and constraints required by the fleet repository.
-- =============================================================================

ALTER TABLE fleet.vehicles
    ADD COLUMN IF NOT EXISTS insurance_expiry date;

CREATE UNIQUE INDEX IF NOT EXISTS vehicles_plate_unique_idx
    ON fleet.vehicles (lower(plate))
    WHERE plate IS NOT NULL;
