CREATE SCHEMA IF NOT EXISTS warehouse;

CREATE TABLE IF NOT EXISTS warehouse.hubs (
    id uuid PRIMARY KEY,
    hub_code text NOT NULL UNIQUE,
    hub_name text NOT NULL,
    latitude double precision NOT NULL,
    longitude double precision NOT NULL,
    capacity_parcels integer NOT NULL CHECK (capacity_parcels >= 0),
    active_parcels integer NOT NULL DEFAULT 0 CHECK (active_parcels >= 0),
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS warehouse.hub_manifest_assignments (
    id uuid PRIMARY KEY,
    hub_id uuid NOT NULL REFERENCES warehouse.hubs(id) ON DELETE CASCADE,
    courier_id uuid NOT NULL,
    order_ids uuid[] NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);
