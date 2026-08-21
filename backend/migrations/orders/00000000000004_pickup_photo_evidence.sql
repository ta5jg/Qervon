ALTER TABLE orders.orders
    ADD COLUMN IF NOT EXISTS pickup_photo_evidence_url text;
