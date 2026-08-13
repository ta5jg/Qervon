CREATE TABLE IF NOT EXISTS tracking.route_breadcrumbs (
    id uuid PRIMARY KEY,
    courier_id uuid NOT NULL,
    latitude double precision NOT NULL,
    longitude double precision NOT NULL,
    speed_kmh double precision NOT NULL,
    battery_level smallint NOT NULL CHECK (battery_level BETWEEN 0 AND 100),
    recorded_at timestamptz NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_route_breadcrumbs_courier_time
    ON tracking.route_breadcrumbs (courier_id, recorded_at DESC);
