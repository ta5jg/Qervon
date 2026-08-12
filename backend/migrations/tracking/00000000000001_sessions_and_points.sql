-- =============================================================================
-- File:           backend/migrations/tracking/00000000000001_sessions_and_points.sql
-- Project:        Qervon
-- Description:    Durable courier tracking sessions and location samples.
-- =============================================================================

CREATE TABLE tracking.sessions (
    id         uuid PRIMARY KEY,
    courier_id uuid NOT NULL REFERENCES couriers.couriers (id),
    status     text NOT NULL,
    started_at timestamptz NOT NULL,
    ended_at   timestamptz,
    CONSTRAINT tracking_sessions_status_check CHECK (status IN ('active', 'ended')),
    CONSTRAINT tracking_sessions_end_check CHECK (
        (status = 'active' AND ended_at IS NULL)
        OR (status = 'ended' AND ended_at IS NOT NULL AND ended_at >= started_at)
    )
);

CREATE UNIQUE INDEX tracking_one_active_session_per_courier
    ON tracking.sessions (courier_id) WHERE status = 'active';

CREATE TABLE tracking.location_points (
    id          uuid PRIMARY KEY,
    courier_id  uuid NOT NULL REFERENCES couriers.couriers (id),
    latitude    double precision NOT NULL CHECK (latitude BETWEEN -90 AND 90),
    longitude   double precision NOT NULL CHECK (longitude BETWEEN -180 AND 180),
    speed_kmh   double precision CHECK (speed_kmh >= 0),
    battery_pct smallint CHECK (battery_pct BETWEEN 0 AND 100),
    recorded_at timestamptz NOT NULL
);

CREATE INDEX tracking_location_points_courier_time_idx
    ON tracking.location_points (courier_id, recorded_at DESC);
