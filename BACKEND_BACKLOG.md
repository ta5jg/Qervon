<!-- =============================================================================
File:           BACKEND_BACKLOG.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-12
Version:        0.1.0

Description:
   Tracks backend domains that are implemented as pure domain/application
   models with unit tests, but are NOT yet wired to a repository, database
   migration, or HTTP route. This is a living engineering note, not a
   governance specification -- keep it short and accurate.

Specification:
   Companion to the Backend Faz-1 Sertleştirme plan.

License:
   Qervon License v1.0 -- see LICENSE in the repository root.
============================================================================= -->

# Backend Backlog (v2)

## Why this file exists

Qervon's backend has a working, tested "delivery vertical slice" (auth, orders,
dispatch, tracking, proof of delivery, billing, notifications). Beyond that
slice, several domains were implemented as pure Rust models with real unit
tests, but were deliberately **not** connected to persistence or the HTTP API
during Backend Faz-1, to keep that phase scoped and shippable.

This file lists exactly what is done vs. missing for each of those domains, so
they are not mistaken for finished features and so picking one up later is a
scoped, well-understood task.

## Status legend

- **Domain model**: real Rust struct/enum with validated constructors and
  behavior, covered by unit tests.
- **Repository**: a `*Repository` trait in `qervon-domain` plus in-memory and
  PostgreSQL adapters in `qervon-infrastructure`.
- **Migration**: a governed SQL migration under `backend/migrations/`.
- **HTTP route**: a handler wired into `backend/apps/api-gateway/src/http.rs`.

## Backlog items

| Domain | File | Domain model | Repository | Migration | HTTP route |
| --- | --- | :---: | :---: | :---: | :---: |
| Warehouse / Cross-Docking Hub | `backend/crates/domain/src/warehouse_hub.rs` | Yes | No | No | No |
| Cold-Chain Temperature Telemetry | `backend/crates/domain/src/cold_chain.rs` | Yes | No | No | No |
| Tax E-Invoicing (VAT draft) | `backend/crates/application/src/tax_invoicing.rs` | Yes | No | No | No |
| Courier Gamification Leaderboard | `backend/crates/application/src/courier_leaderboard.rs` | Yes | No | No | No |
| GPS Route Playback (breadcrumbs) | `backend/crates/domain/src/route_history.rs` | Yes | No | No | No |
| Field Service Appointment Scheduling | `backend/crates/application/src/field_service.rs` | Yes | No | No | No |
| Multi-Currency Exchange | `backend/crates/application/src/currency_exchange.rs` | Yes | No | No | No |

Each file above carries a `// STATUS: v2 backlog ...` comment right after its
header, pointing back to this document.

The following domains were promoted out of this backlog during **Mobil Faz-2.1
(destekleyici backend API'leri)**: Courier Wallet
(`backend/crates/domain/src/courier_wallet.rs`), Promo Coupon
(`backend/crates/application/src/promo_coupon.rs`, now backed by the persisted
`Coupon` entity in `backend/crates/domain/src/coupon.rs`), and Customer
Ratings & Support Tickets (`backend/crates/domain/src/customer_feedback.rs`).
Each now has a full repository (memory + PostgreSQL), migration, and HTTP
routes — see "Faz-2.1 scope boundaries" below for what is still simplified
within them.

## Promoting an item out of the backlog

When picking one of these up, follow the same pattern already used by the
shipped domains (e.g. Fleet in `backend/crates/application/src/fleet_service.rs`,
`backend/modules/fleet`, and `backend/crates/infrastructure/src/postgres.rs`):

1. Add a `*Repository` trait to `backend/crates/domain/src/repository.rs`.
2. Implement it for both `InMemoryStore` (`memory.rs`) and Postgres
   (`postgres.rs`), plus a governed migration under `backend/migrations/`.
3. Wire it into `AppState` (`backend/apps/api-gateway/src/state.rs`).
4. Add HTTP routes in `http.rs` following the existing tenant-scoping pattern
   (`require_operational_access` + `find_*_tenant` checks).
5. Add HTTP-level tests in `backend/apps/api-gateway/tests/api_flow.rs`,
   including a tenant-isolation test.
6. Remove the `// STATUS: v2 backlog` comment and this table row.

## Explicitly out of scope for Backend Faz-1

- Mobile app changes (separate phase).
- Deepening AI Dispatcher/ETA beyond the Fraud Guard wiring done in Faz-1.
- The dead `web/` React scaffold and governance-document labeling (separate,
  low-cost phases).

## Faz-2.1 scope boundaries (mobile-supporting backend APIs)

Faz-2.1 built the backend APIs a native mobile app needs (OTP login, courier
wallet, ratings/support tickets, coupons, order payment method, native push
registration). Every one of these is a **real, tested, persisted** feature —
none are placeholders — but three of them have a deliberately limited edge
that requires a real third-party provider we do not have credentials for in
this environment. Each is called out in code comments at its integration
point; this section is the single place that lists all three together.

- **OTP phone login has no real SMS provider.** `OtpService`
  (`backend/crates/application/src/otp_service.rs`) generates a
  cryptographically random 6-digit code, hashes it, persists the challenge,
  and verifies it with attempt/expiry limits — that whole flow is real. What
  is missing is delivery: `POST /v1/auth/otp/request` never calls an SMS API.
  On in-memory (local/dev) storage the raw code is returned in the response
  body as `dev_code` purely for local testing; on PostgreSQL storage it is
  only written to the server log (never the HTTP response) and only the
  fact that a code was issued is logged, not the code itself. Wiring a real
  provider (e.g. Twilio) means replacing the `tracing::info!` call in
  `auth_otp_request` (`backend/apps/api-gateway/src/http.rs`) with an actual
  API call, and removing the `dev_code` field entirely once that exists.
- **Order payment methods do not move money.** `Order.payment_method`
  (`Cash | Card | Qr | Wallet`) and `Order.payment_collected`
  (`backend/crates/domain/src/order.rs`) are real, persisted fields with a
  real state machine (`mark_payment_collected` requires a chosen method and
  cannot be confirmed twice). But `Card`/`Qr`/`Wallet` only ever record the
  customer's chosen method — no payment gateway (iyzico, Stripe, etc.) is
  integrated, and no card data is collected or transmitted (this also keeps
  the platform outside PCI-DSS scope for now). Only `Cash` has a real-world
  action behind it: the courier's own confirmation that they physically
  collected the amount, via `payment_collected` on
  `POST /v1/courier/orders/{id}/deliver`.
- **Native push registration does not send anything.** `DevicePushToken`
  (`backend/crates/domain/src/device_push_token.rs`) and
  `POST /v1/push/devices` / `DELETE /v1/push/devices/{id}` are a real,
  tenant-agnostic, per-user registry of iOS/Android device tokens with
  idempotent re-registration and ownership-scoped deletion. Nothing reads
  from this table yet: there is no APNs/FCM integration, so registering a
  device does not cause any push notification to actually arrive. This
  mirrors the existing browser web-push split already in the codebase
  (`notifications.web_push_subscriptions` registration vs. the delivery
  logic in `backend/apps/worker`) — the next step is a worker job that reads
  `notifications.device_push_tokens`, calls Apple/Google's push APIs with
  real credentials, and marks unreachable tokens invalid.

None of the above blocks local development or testing: every code path is
exercised by an in-memory integration test in
`backend/apps/api-gateway/tests/api_flow.rs`. They block is only *real-world*
delivery (SMS/push) and *real-world* money movement (card/QR/wallet
payments), which both require credentials/contracts this environment
intentionally does not have.

## Faz-2.2 scope boundaries (native iOS Courier app)

Faz-2.2 added a real offer→accept/reject assignment flow (`Assignment.status
== Offered`, `offer_for_order`/`accept_offer`/`reject_offer` in
`backend/crates/application/src/dispatch_service.rs`, 45s TTL) and built the
native iOS Courier app against it and the rest of the courier-facing API
surface. One edge is worth calling out explicitly:

- **`POST /v1/courier/orders/{id}/pickup` records no proof-of-pickup.** It
  only transitions the order to `InTransit`; there is no equivalent to
  `POST /v1/courier/orders/{id}/deliver`'s evidence fields
  (`recipient_name`, `qr_barcode_verified`, `digital_signature_base64`,
  `photo_evidence_url`) for the pickup leg. The iOS app's Pickup screen is
  therefore a single confirmation step rather than a QR/photo capture that
  would have no server-side effect. Adding pickup evidence fields (mirroring
  the delivery ones) is a reasonable future addition if operations need it.
- **`photo_evidence_url` upload is now real (added 2026-08-13), but
  local-filesystem, not cloud object storage.**
  `POST /v1/courier/orders/{id}/photo-evidence` accepts a real multipart
  JPEG/PNG upload, saves it under `AppState.uploads_dir` (configured via
  `QERVON_UPLOADS_DIR`), and returns a URL the client passes back as
  `photo_evidence_url` — both the iOS and Android courier apps now call
  this before delivering. It is *not* S3-compatible object storage (no
  such credential exists in this environment) — the uploads directory
  must be a persistent, backed-up path on the production VPS (see
  QAS-000014). A future swap to a real object store would only need to
  change this one endpoint's storage backend, not the client-side
  contract (`{"url": "..."}`).

## Faz-2.3 scope boundaries (native iOS Customer app + delivery pricing)

Faz-2.3 added a real, distance-based delivery pricing engine
(`qervon_domain::DeliveryPricing`, `PricingService` in
`backend/crates/application/src/pricing_service.rs`) and built the native
iOS Customer app against it and the rest of the customer-facing API surface.
`POST /v1/customer/orders` no longer accepts a client-supplied fare at
all — the server always computes it itself from `PricingService::quote_fare`
before applying any coupon, so a client can never manipulate what it is
charged. Two edges are worth calling out explicitly:

- **Tenant pricing admin UI (added 2026-08-13).** The admin panel
  (`backend/apps/api-gateway/static/index.html`, "Fiyatlandırma" tab) now
  has a real screen backed directly by `GET`/`PUT /v1/pricing`: it loads
  the tenant's current fare formula, lets an `Admin`/`SuperAdmin` edit and
  save it (major-unit inputs like `10.00`, converted to/from minor units
  client-side), shows a live example-fare preview computed with the same
  formula the backend uses, and disables the form (read-only) for every
  other role — matching the backend's own `Admin`/`SuperAdmin`-only
  enforcement rather than just hiding a client-side error. An unconfigured
  tenant is still not blocked: it gets a real, documented default (base
  ₺10 + ₺2.50/km, ₺15 minimum, see `qervon_domain::DEFAULT_BASE_FARE_MINOR`
  and friends).
- **No coupon preview endpoint.** The customer app can see the *base* fare
  quote (`GET /v1/customer/fare-quote`) before creating an order, but there
  is no way to preview a coupon's discount without actually submitting the
  order — `CouponService::apply_to_fare` redeems on use, so a true
  no-side-effect preview would need a separate read-only code path. This was
  kept out of scope to avoid touching coupon redemption semantics.

## Automatic re-offer cascade (added 2026-08-13)

`POST /v1/courier/orders/{id}/reject` and a courier's own expired-offer
discovery (`GET /v1/courier/me/offer`) now automatically re-offer a still-
`Pending` order to the next-best available courier in the same tenant,
excluding every courier already offered-and-rejected/expired for that order
(`Assignment.excluded_courier_ids`, `DispatchService::reoffer_from_candidates`,
see `docs/qls/QLS-000003-dispatch-domain.md`). This closes what was
previously the "no automatic re-offer cascade" gap noted in the Faz-2.2 and
Faz-2.3 sections above. It cascades one step at a time, lazily, as each
courier responds or times out — there is still no synchronous "try every
candidate in a loop" path, and if every available courier in the tenant has
already been tried, the order falls back to `Pending` for an operator to
resolve manually via `POST /v1/orders/{id}/assign`, exactly as before.

## Faz-2.4 scope boundaries (native Android Courier + Customer apps)

Faz-2.4 built native Android (`app-courier`, `app-customer` under
`mobile/android/`) against the exact same backend contract as the iOS apps
— a purely client-side phase, no backend changes. All the Faz-2.1/2.2/2.3
boundary notes above apply identically to the Android clients (no
SMS/real-payment integration, no pricing admin UI); the automatic
re-offer cascade and the real local-filesystem photo-evidence upload
above are both backend-driven behaviors the Android client already uses
the same way the iOS client does. One boundary is **Android-specific and
wider than iOS's**:

- **No native push (FCM) integration at all**, whereas the iOS app at
  least captures a real (if practically unusable in this environment) APNs
  device token and posts it to `POST /v1/push/devices`. Firebase Cloud
  Messaging requires a `google-services.json` Firebase project credential
  at *build* time; without it, the `com.google.gms.google-services` Gradle
  plugin fails the build outright (unlike iOS, where a missing Push
  entitlement only fails silently at *runtime*). Since no such credential
  exists in this environment, the Android apps do not include the FCM SDK
  or call `POST /v1/push/devices` at all in this phase.   The endpoint itself
  is unchanged and ready for a future Android client that does have a real
  Firebase project.

## Web platform boundary notes (vanilla HTML/JS under api-gateway/static/)

The empty `web/` React/Vite scaffold (43 header-only files, no dependencies,
could not build) was deleted; `backend/apps/api-gateway/static/` — the
vanilla HTML/CSS/JS pages already served at `/`, `/customer.html`, the
mobile simulators, `/login`, and `/setup` — was kept as the project's
official web platform. A security + functionality pass fixed a real XSS gap,
pinned an unpinned CDN script version, and replaced several fabricated
demo values (fake wallet balance, fake ratings) with real API-backed data.
Two decorative UI elements were removed rather than wired up, because no
backend support exists for them yet:

- **Bulk Excel/CSV order import** (`customer.html`'s "Toplu Excel Yükle" tab)
  has no backend endpoint at all — there is no CSV/XLSX parsing route.
  Building one would need a multipart upload endpoint plus a parser
  (`calamine`/`csv` crate) reusing `OrderService::create` per row.
- **Browser-based camera QR/barcode scanning and photo capture**
  (`mobile-courier.html`'s POD tab) were removed as non-functional buttons
  rather than implemented — a real implementation would need the
  `BarcodeDetector` Web API (limited browser support) or a JS QR library
  plus `getUserMedia()`, which is a meaningfully different scope than the
  simple checkbox this simulator now honestly offers. The native iOS
  (VisionKit) and Android (ML Kit) apps already do this for real.
