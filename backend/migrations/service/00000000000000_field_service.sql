CREATE SCHEMA IF NOT EXISTS service;

CREATE TABLE IF NOT EXISTS service.field_service_appointments (
    id uuid PRIMARY KEY,
    customer_id uuid NOT NULL,
    technician_id uuid,
    service_type text NOT NULL,
    appointment_date date NOT NULL,
    slot_window text NOT NULL CHECK (slot_window IN ('Morning', 'Afternoon', 'Evening')),
    is_confirmed boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_field_service_customer_date
    ON service.field_service_appointments (customer_id, appointment_date DESC);
