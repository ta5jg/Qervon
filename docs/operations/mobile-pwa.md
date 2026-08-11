# Mobile PWA Delivery

The customer and courier terminals expose a web manifest and service worker.
They can be installed as standalone mobile web applications and retain the
authenticated shell for temporary network loss. API writes and live tracking
are never cached, so stale operational data is not replayed as current data.

Both terminals use live regions for status changes. Delivery proof requires a
labeled recipient field and an explicit QR confirmation from the courier.
