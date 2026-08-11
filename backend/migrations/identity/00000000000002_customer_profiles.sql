-- =============================================================================
-- File:           backend/migrations/identity/00000000000002_customer_profiles.sql
-- Description:    Durable customer profiles and address books.
-- =============================================================================

CREATE TABLE identity.customer_profiles (
    id             uuid PRIMARY KEY,
    user_id        uuid NOT NULL UNIQUE REFERENCES identity.users (id) ON DELETE CASCADE,
    company_name   text,
    tax_id         text,
    loyalty_points bigint NOT NULL DEFAULT 0 CHECK (loyalty_points >= 0),
    created_at     timestamptz NOT NULL
);

CREATE TABLE identity.customer_addresses (
    id                  uuid PRIMARY KEY,
    customer_profile_id uuid NOT NULL REFERENCES identity.customer_profiles (id) ON DELETE CASCADE,
    label               text NOT NULL CHECK (length(trim(label)) > 0),
    latitude            double precision NOT NULL,
    longitude           double precision NOT NULL,
    full_address        text NOT NULL CHECK (length(trim(full_address)) > 0),
    is_default          boolean NOT NULL DEFAULT false,
    CONSTRAINT customer_addresses_latitude_check CHECK (latitude BETWEEN -90 AND 90),
    CONSTRAINT customer_addresses_longitude_check CHECK (longitude BETWEEN -180 AND 180)
);

CREATE UNIQUE INDEX customer_addresses_one_default_idx
    ON identity.customer_addresses (customer_profile_id)
    WHERE is_default;

CREATE INDEX customer_addresses_profile_idx
    ON identity.customer_addresses (customer_profile_id, id);
