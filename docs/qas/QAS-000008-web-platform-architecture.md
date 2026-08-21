<!-- =============================================================================
File:           docs/qas/QAS-000008-web-platform-architecture.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  The web platform's real architecture: vanilla HTML/CSS/JS served
  directly by the Rust backend. Supersedes the originally-planned
  React/TypeScript platform (ADR-000004).

Specification:
  ADR-000004, QAS-000005.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000008 — Web Platform Architecture

**Status: Implemented.** See ADR-000004 for the full history of how this
diverged from the originally-planned React/TypeScript platform.

## Structure

Every web page is a single `.html` file under
`backend/apps/api-gateway/static/`, embedded into the `api-gateway`
binary at compile time via `include_str!` and served by a dedicated Axum
handler (`serve_dashboard`, `serve_customer_portal`, ...). There is no
separate build step, bundler, or `node_modules` — the page ships inside
the same binary as the API it calls.

| File | Route | Audience |
| --- | --- | --- |
| `index.html` | `/`, `/index.html` | Tenant admin dashboard |
| `customer.html` | `/customer`, `/customer.html` | Corporate customer portal |
| `mobile-customer.html` | `/mobile-customer.html` | Customer mobile-web simulator |
| `mobile-courier.html` | `/mobile-courier.html` | Courier mobile-web simulator |
| `login.html` | `/login` | Login + customer self-registration |
| `setup.html` | `/setup` | First-time platform bootstrap |

Each page is self-contained: its own `<style>` block, its own small
vanilla-JS `api()`/`adminApi()` fetch wrapper, no shared JS module system
across pages (the closest thing to a shared piece is
`qervon-client.js`, a small standalone helper not currently wired as a
`<script src>` on any page).

## Per-page conventions (established during the 2026-08-12 audit)

- **`escapeHtml()`** — every page that renders API data via `innerHTML`
  defines this helper and must use it on every string field that could
  contain user input (an address label, a display name, an email). This
  is the fix for the one real XSS gap found in this codebase
  (`mobile-customer.html`'s order-history render was missing it on
  pickup/dropoff labels).
- **CSRF** — every page's fetch wrapper attaches `X-Csrf-Token` from the
  `qervon_csrf_token` cookie on every request (see QAS-000004); harmless
  on GETs, required on mutating requests.
- **Fare display** — no page hardcodes a price. Every order-creation flow
  calls `GET /v1/customer/fare-quote` for a live estimate and displays
  whatever `POST /v1/customer/orders` actually returns as the
  authoritative fare — the backend, never the client, decides the charge.
- **Third-party CDN scripts** (`Leaflet`, `lucide`) are pinned to an exact
  version, never `@latest` — an unpinned CDN script is a supply-chain
  risk (a compromised or unexpectedly-changed "latest" build runs with
  full page privileges).

## Bulk CSV order import

The customer portal implements bulk creation through
`POST /v1/customer/orders/bulk`. The browser uploads an RFC 4180 UTF-8 CSV
body (maximum 1 MB / 100 rows) and renders the returned client reference,
order number, authoritative fare and status for every created order. The
downloadable template is the canonical column contract.

Security and consistency rules are enforced server-side:

- customer and tenant ownership come only from the authenticated session;
- client-supplied fare, currency, customer or unknown columns are rejected;
- all rows, coordinates, phone numbers and tenant fare quotes are validated
  before the first order is created;
- `reference` must be unique within the uploaded file;
- QR payment stays disabled, matching single-order creation.

Native `.xlsx` parsing is deliberately not shipped. Excel, Numbers and other
spreadsheet users save the provided template as CSV before upload.

## What is deliberately not built (rather than faked)

- **Browser-based camera QR/photo capture** — `mobile-courier.html`'s POD
  tab offers a manual "QR doğrulandı" checkbox instead of a fake
  camera-scan button; a real implementation would need `getUserMedia()`
  plus a JS barcode-detection library, which the native iOS/Android apps
  already do for real (see QAS-000007).
- **A React/TypeScript SPA** — see ADR-000004. If ever revisited, this
  would be a larger undertaking than the entire mobile phase, since it
  means re-implementing pages that already work.

## References

- [ADR-000004](../adr/ADR-000004-use-react-typescript-for-web.md) (the full decision history), [QAS-000004](QAS-000004-security-architecture.md) (security controls
  these pages rely on), [QAS-000005](QAS-000005-api-integration-standard.md) (the API contract they call).
- Root [README.md](../../README.md) "Web Platformu Kararı" section.

---

# Revision History

| Version | Date | Description |
| --------- | ------ | ------------- |
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs, describing a React/TypeScript platform. |
| 0.2.0 | 2026-08-12 | Rewritten to describe the real vanilla HTML/JS architecture. |
| 0.3.0 | 2026-08-12 | Added the per-page conventions established during the security/functionality audit. |
| 0.4.0 | 2026-08-22 | Added the authenticated, server-priced bulk CSV order workflow and its safety boundary. |
