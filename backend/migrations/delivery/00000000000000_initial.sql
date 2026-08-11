-- =============================================================================
-- File:           backend/migrations/delivery/00000000000000_initial.sql
-- Project:        Qervon
-- Description:    Immutable proof-of-delivery evidence records.
-- =============================================================================

CREATE SCHEMA IF NOT EXISTS delivery;

CREATE TABLE delivery.proofs_of_delivery (
    id                        uuid PRIMARY KEY,
    order_id                  uuid NOT NULL UNIQUE,
    courier_id                uuid NOT NULL,
    recipient_name            text NOT NULL,
    qr_barcode_verified       boolean NOT NULL DEFAULT false,
    digital_signature_base64  text,
    photo_evidence_url        text,
    delivered_at              timestamptz NOT NULL,
    CONSTRAINT proofs_recipient_name_check CHECK (length(trim(recipient_name)) > 0),
    CONSTRAINT proofs_evidence_check CHECK (
        qr_barcode_verified
        OR digital_signature_base64 IS NOT NULL
        OR photo_evidence_url IS NOT NULL
    )
);

CREATE INDEX proofs_of_delivery_courier_idx
    ON delivery.proofs_of_delivery (courier_id, delivered_at DESC);
