CREATE TABLE IF NOT EXISTS delivery.cold_chain_telemetry (
    id uuid PRIMARY KEY,
    order_id uuid NOT NULL,
    sensor_id text NOT NULL,
    temperature_celsius double precision NOT NULL,
    humidity_percent double precision NOT NULL,
    min_allowed_temp double precision NOT NULL,
    max_allowed_temp double precision NOT NULL,
    is_violation boolean NOT NULL,
    recorded_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_cold_chain_order_time
    ON delivery.cold_chain_telemetry (order_id, recorded_at DESC);
