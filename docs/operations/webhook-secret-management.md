# Webhook Secret Management

Production webhook delivery requires `QERVON_WEBHOOK_ENCRYPTION_KEY`, a
32-byte base64 key supplied only through `/etc/qervon/qervon.env`. Webhook
secrets must be encrypted before persistence; hashes are retained only for
verification and must never be used as signing material. Rotate the master key
through a decrypt-and-reencrypt migration, never by replacing it in place.

The persisted value is `12-byte AES-GCM nonce || ciphertext`. The worker
decrypts it only in memory to create an `sha256=<hex>` HMAC over the exact JSON
bytes stored for that delivery. Secrets and decrypted values must never appear in
logs, API responses, dead-letter records or diagnostics.
