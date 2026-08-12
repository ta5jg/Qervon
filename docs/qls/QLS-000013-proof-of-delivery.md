<!-- =============================================================================
File:           docs/qls/QLS-000013-proof-of-delivery.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  What proof-of-delivery evidence the backend actually accepts, and what
  each client platform actually captures.

Specification:
  QAS-000007, QLS-000002.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000013 — Proof of Delivery

**Status: Implemented (delivery only — no pickup evidence).**

## Backend contract

`POST /v1/courier/orders/{id}/deliver` requires **at least one** of:
`qr_barcode_verified: bool`, `digital_signature_base64: Option<String>`,
`photo_evidence_url: Option<String>` — plus always `recipient_name` and,
for cash orders, `payment_collected: bool`. Supplying none of the three
evidence fields is a `422 Unprocessable Entity`, not a silently-accepted
delivery.

## No pickup evidence

`POST /v1/courier/orders/{id}/pickup` takes no body at all — it's a
single state transition, `CourierAssigned → InTransit`, with no evidence
fields to submit. Every client's "pickup" screen is therefore a single
confirmation tap, not a QR/photo capture that would have no server-side
effect — this is a deliberate honesty-driven design choice, not an
oversight (see QAS-000001).

## What each platform actually captures

| Evidence | iOS | Android | Web (`mobile-courier.html`) |
| --- | --- | --- | --- |
| QR/barcode scan | Real (VisionKit `DataScannerViewController`; Simulator falls back to a manual toggle, no camera there) | Real (ML Kit Barcode Scanning + CameraX) | Manual checkbox only — no camera API used (see QAS-000008) |
| Digital signature | Real (`PKCanvasView`-based pad, real base64 PNG) | Real (Compose `Canvas` pad, real base64 PNG) | Not offered |
| Photo | Real capture (device camera) and real upload | Real capture (CameraX) and real upload | Not offered |

## `photo_evidence_url` upload path (added 2026-08-13)

Both mobile apps capture a real photo, then upload it via
`POST /v1/courier/orders/{id}/photo-evidence` (a real multipart JPEG/PNG
upload, courier- and order-ownership-checked the same way `deliver`/
`pickup` are) before calling `deliver`, and pass the returned URL as
`photo_evidence_url`. The uploaded file is served back via
`GET /v1/uploads/delivery-photos/{order_id}/{filename}`, gated to signed-in
members of the tenant that owns the order.

This is real, working persistence — but a local filesystem directory
(`AppState.uploads_dir`, configured via `QERVON_UPLOADS_DIR`), not a cloud
object store; no such credential exists in this environment. The uploads
directory must be a persistent, backed-up path on the production VPS (see
QAS-000014's deployment runbook). A future swap to an S3-compatible bucket
with presigned upload URLs would only change this one endpoint's storage
backend, not the `{"url": "..."}` client-side contract.

## References

- QAS-000007 (mobile platform, the per-platform capture detail),
  QLS-000002 (order lifecycle this evidence completes), BACKEND_BACKLOG.md.

---

# Revision History

| Version | Date | Description |
| --------- | ------ | ------------- |
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real backend contract and per-platform capture capability table. |
| 0.3.0 | 2026-08-13 | The photo-evidence upload gap is closed: real multipart upload + serve endpoints, wired into both mobile clients. |
