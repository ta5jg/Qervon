# Proof of Delivery Persistence

Delivery evidence is stored through the `ProofOfDeliveryRepository` port. Local
and test environments use the in-memory adapter; PostgreSQL deployments use
`delivery.proofs_of_delivery` after the governed migration is applied.

There is exactly one POD record per order. The database enforces a non-empty
recipient name and requires at least one evidence signal: QR/barcode
verification, digital signature, or a photo evidence URL. Customer retrieval
remains tenant- and order-owner-scoped by the existing API authorization path.

Apply the migration with the normal release flow before enabling the PostgreSQL
adapter. POD content can include personal data, so operational logs must not
record recipient names, signatures, evidence URLs, or request bodies.
