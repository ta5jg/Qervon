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
during Backend Faz-1, to keep that phase scoped and shippable. All of them
were promoted out of backlog status during the `full-vision-campaign` and its
2026-08-13 follow-up hardening pass (see "Backlog closure" below); this file
now exists mainly as the historical record of what that promotion involved,
plus the still-honest boundary notes further down (Faz-2.1 through Faz-2.4,
web platform).

## Status legend

- **Domain model**: real Rust struct/enum with validated constructors and
  behavior, covered by unit tests.
- **Repository**: a `*Repository` trait in `qervon-domain` plus in-memory and
  PostgreSQL adapters in `qervon-infrastructure`.
- **Migration**: a governed SQL migration under `backend/migrations/`.
- **HTTP route**: a handler wired into `backend/apps/api-gateway/src/http.rs`,
  tenant-scoped the same way every other operational endpoint is
  (`require_operational_access` + a `tenant_id` check, either stored on the
  entity itself or resolved via `TenantRepository::find_courier_tenant`).

## Backlog closure (2026-08-13)

Every domain below is now **fully wired**: real repository, real migration
(or, for the two stateless calculators, no persistence needed because there
is no entity to store), real tenant-scoped HTTP route, and HTTP-level tests
including an explicit tenant-isolation test in
`backend/apps/api-gateway/tests/api_flow.rs`. The Postgres adapters were also
verified against a real local PostgreSQL instance via
`scripts/test-postgres-integration.sh` (see the round-trip assertions added to
`backend/crates/infrastructure/tests/postgres_repositories.rs`), not just the
in-memory adapter that the HTTP-level tests exercise.

| Domain | File | Domain model | Repository | Migration | HTTP route |
| --- | --- | :---: | :---: | :---: | :---: |
| Warehouse / Cross-Docking Hub | `backend/crates/domain/src/warehouse_hub.rs` | Yes | Yes (Postgres) | Yes | Yes |
| Cold-Chain Temperature Telemetry | `backend/crates/domain/src/cold_chain.rs` | Yes | Yes (Postgres) | Yes | Yes |
| Tax E-Invoicing (VAT draft) | `backend/crates/application/src/tax_invoicing.rs` | Yes | N/A — stateless calculator | N/A | Yes |
| Courier Gamification Leaderboard | `backend/crates/application/src/courier_leaderboard.rs` | Yes | N/A — read model over Order/CustomerRating | N/A | Yes |
| GPS Route Playback (breadcrumbs) | `backend/crates/domain/src/route_history.rs` | Yes | Yes (Postgres) | Yes | Yes |
| Field Service Appointment Scheduling | `backend/crates/domain/src/field_service.rs` | Yes | Yes (Postgres) | Yes | Yes |
| Multi-Currency Exchange | `backend/crates/application/src/currency_exchange.rs` | Yes | N/A — stateless calculator | N/A | Yes |

Notes on the two "N/A" rows, so they are not mistaken for oversights:

- **Tax E-Invoicing and Multi-Currency Exchange** compute a result from
  request parameters (a VAT draft, a converted amount) with no state that
  outlives the request — there is nothing to persist. This mirrors how
  `PricingService::quote_fare` (delivery pricing preview) already worked
  before this pass.
- **Courier Leaderboard** is a derived read model: every input (completed
  deliveries, on-time rate, average rating) is computed live from the
  existing `OrderRepository` and `CustomerRatingRepository` in
  `GET /v1/couriers/leaderboard`, rather than duplicated into a new table
  that could drift out of sync with the orders/ratings it summarizes.
  "On-time" is defined as delivered within 60 minutes of order creation — a
  real, timestamp-derived measure, not a fabricated value.

`FieldServiceAppointment` moved from `qervon-application` to `qervon-domain`
as part of this pass (`backend/crates/domain/src/field_service.rs`), so its
repository trait could live in `repository.rs` alongside every other one — a
domain crate cannot depend on the application crate that used to own it.

The following domains were promoted out of this backlog earlier, during
**Mobil Faz-2.1 (destekleyici backend API'leri)**: Courier Wallet
(`backend/crates/domain/src/courier_wallet.rs`), Promo Coupon
(`backend/crates/application/src/promo_coupon.rs`, now backed by the persisted
`Coupon` entity in `backend/crates/domain/src/coupon.rs`), and Customer
Ratings & Support Tickets (`backend/crates/domain/src/customer_feedback.rs`).
Each has a full repository (memory + PostgreSQL), migration, and HTTP
routes — see "Faz-2.1 scope boundaries" below for what is still simplified
within them.

## Promoting a future item out of the backlog

There is nothing left in this backlog as of 2026-08-13, but if a new domain
is added as a pure model first, follow the same pattern used by every shipped
domain above (e.g. Fleet in
`backend/crates/application/src/fleet_service.rs`, `backend/modules/fleet`,
and `backend/crates/infrastructure/src/postgres.rs`):

1. Add a `*Repository` trait to `backend/crates/domain/src/repository.rs`.
2. Implement it for both `InMemoryStore` (`memory.rs`) and Postgres
   (`postgres.rs`), plus a governed migration under `backend/migrations/`.
3. Wire it into `AppState` (`backend/apps/api-gateway/src/state.rs`).
4. Add HTTP routes in `http.rs` following the existing tenant-scoping pattern
   (`require_operational_access` + `find_*_tenant` checks, or a `tenant_id`
   column checked directly on the entity).
5. Add HTTP-level tests in `backend/apps/api-gateway/tests/api_flow.rs`,
   including a tenant-isolation test, and round-trip the new Postgres
   repository in `backend/crates/infrastructure/tests/postgres_repositories.rs`.
6. Remove the `// STATUS: v2 backlog` comment and add the domain to the table
   above (or delete the table if it becomes empty again, as happened here).

## Explicitly out of scope for Backend Faz-1

- Mobile app changes (separate phase).
- Deepening AI Dispatcher/ETA beyond the Fraud Guard wiring done in Faz-1.
- The dead `web/` React scaffold and governance-document labeling (separate,
  low-cost phases).

## Faz-2.1 scope boundaries (mobile-supporting backend APIs)

Faz-2.1 built the backend APIs a native mobile app needs (OTP login, courier
wallet, ratings/support tickets, coupons, order payment method, native push
registration). Every one of these is a **real, tested, persisted** feature —
none are placeholders. Three of them additionally touch a real third-party
provider (SMS, a payment gateway, APNs/FCM); the `full-vision-campaign`'s
ops-hardening step (see `docs/operations/full-vision-acceptance-report.md`)
added a real, pluggable outbound HTTP client for each — configurable via
`QERVON_SMS_PROVIDER_URL`/`_TOKEN`, `QERVON_PAYMENT_GATEWAY_URL`/`_TOKEN`, and
`QERVON_PUSH_PROVIDER_URL`/`_TOKEN` — so what remains environment-dependent is
**only the actual third-party account and endpoint URL**, not the call site
itself. This is proven end-to-end (real HTTP request, real bearer-token
header, real request body) in
`backend/apps/api-gateway/tests/outbound_providers.rs`, which points each
provider URL at a local test server and asserts on what arrives.

- **OTP phone login delivery.** `OtpService`
  (`backend/crates/application/src/otp_service.rs`) generates a
  cryptographically random 6-digit code, hashes it, persists the challenge,
  and verifies it with attempt/expiry limits — that whole flow is real.
  `POST /v1/auth/otp/request` calls `deliver_otp_sms`
  (`backend/apps/api-gateway/src/http.rs`), which POSTs `{phone, message}` to
  `QERVON_SMS_PROVIDER_URL` with a bearer token if configured. When no URL is
  configured (the default in this environment, since there is no real SMS
  account here), delivery is a no-op and, on in-memory (local/dev) storage
  only, the raw code is returned in the response body as `dev_code` purely
  for local testing; on PostgreSQL storage it is only written to the server
  log, never the HTTP response. A delivery failure is logged as a warning but
  never fails the request, so a flaky provider cannot lock a user out of
  requesting a new code.
- **Order payment methods.** `Order.payment_method`
  (`Cash | Card | Qr | Wallet`) and `Order.payment_collected`
  (`backend/crates/domain/src/order.rs`) are real, persisted fields with a
  real state machine (`mark_payment_collected` requires a chosen method and
  cannot be confirmed twice). `POST /v1/payments/charge` forwards
  `{order_id, amount_minor, currency, method}` to `QERVON_PAYMENT_GATEWAY_URL`
  with a bearer token when configured, and reports `"accepted"`/`"failed"`
  based on the gateway's HTTP response; with no URL configured it returns
  `"simulated"` rather than silently pretending money moved. This endpoint is
  not yet called automatically from order creation for `Card`/`Qr`/`Wallet`
  orders — an operator or client must invoke it explicitly — and no card
  data is collected or transmitted (keeping the platform outside PCI-DSS
  scope). Only `Cash` has a real-world action wired into the delivery flow
  itself: the courier's own confirmation that they physically collected the
  amount, via `payment_collected` on
  `POST /v1/courier/orders/{id}/deliver`.
- **Native push dispatch — iOS/APNs is real as of 2026-08-16; Android/FCM is
  not.** `DevicePushToken`
  (`backend/crates/domain/src/device_push_token.rs`) and
  `POST /v1/push/devices` / `DELETE /v1/push/devices/{id}` are a real,
  tenant-agnostic, per-user registry of device tokens, each tagged with an
  `app_variant` (`courier` | `customer`) so a push provider knows which iOS
  bundle id's `apns-topic` to address — the two apps have distinct bundle
  ids. `backend/apps/api-gateway/src/apns.rs` is a real APNs HTTP/2 client:
  it builds an ES256-signed provider JWT (cached ~45 minutes, per Apple's
  guidance) and POSTs an alert payload to Apple's sandbox/production gateway.
  `POST /v1/push/native/dispatch` sends every iOS device token through it
  when `APNS_TEAM_ID`/`APNS_KEY_ID`/`APNS_PRIVATE_KEY_PATH`/
  `APNS_BUNDLE_ID_COURIER`/`APNS_BUNDLE_ID_CUSTOMER` are configured (see
  `.env.example`); with any of those missing, iOS tokens fall back to the
  same generic `QERVON_PUSH_PROVIDER_URL` webhook Android tokens still use
  (or `"simulated"` if that is unset too). Two domain events now call this
  automatically rather than requiring an explicit trigger: a courier
  receiving a new job offer (`offer_for_tenant`/`reoffer_for_tenant`) and a
  customer's order being delivered (`deliver_order`/`courier_deliver_order`)
  — both fire-and-forget (`notify_user_push`), so a push failure never turns
  an otherwise-successful assignment or delivery into an HTTP error.
  Android/FCM remains entirely unwired — see the Faz-2.4 section below for
  why (a `google-services.json` Firebase credential is required at Android
  *build* time, which this environment does not have).

None of the above blocks local development or testing: every code path,
including the outbound HTTP call itself, is exercised by an integration test
(`backend/apps/api-gateway/tests/outbound_providers.rs` for the provider
call-outs; `backend/apps/api-gateway/tests/api_flow.rs` for the rest). What
blocks is only *real-world* delivery (an actual SMS arriving on a phone, a
real push notification arriving on a device) and *real-world* money movement
(card/QR/wallet payments), which require credentials/contracts and, for push,
a protocol-shaped provider, that this environment intentionally does not
have.

## Faz-2.2 scope boundaries (native iOS Courier app)

Faz-2.2 added a real offer→accept/reject assignment flow (`Assignment.status
== Offered`, `offer_for_order`/`accept_offer`/`reject_offer` in
`backend/crates/application/src/dispatch_service.rs`, 45s TTL) and built the
native iOS Courier app against it and the rest of the courier-facing API
surface. One edge is worth calling out explicitly:

- **Pickup photo evidence is implemented (closed 2026-08-21).**
  `POST /v1/courier/orders/{id}/pickup` requires
  `pickup_photo_evidence_url`; both native courier apps capture and upload
  a real image before the transition. Empty evidence is rejected with
  `422 Unprocessable Entity`, and upload failure leaves the order assigned.
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
One decorative UI element was removed rather than wired up, because no
browser support exists for it yet. The former bulk-order gap is closed:

- **Bulk CSV order import** is implemented at
  `POST /v1/customer/orders/bulk` and in `customer.html`. It accepts at most
  100 RFC 4180 rows / 1 MB, rejects customer and fare fields, validates the
  complete file before the first write, derives ownership from the signed
  customer session, and computes every fare from tenant pricing. The portal
  provides a downloadable UTF-8 CSV template and a per-reference result list.
  Native `.xlsx` parsing remains out of scope; spreadsheet users export the
  supplied template as CSV.
- **Browser-based camera QR/barcode scanning and photo capture**
  (`mobile-courier.html`'s POD tab) were removed as non-functional buttons
  rather than implemented — a real implementation would need the
  `BarcodeDetector` Web API (limited browser support) or a JS QR library
  plus `getUserMedia()`, which is a meaningfully different scope than the
  simple checkbox this simulator now honestly offers. The native iOS
  (VisionKit) and Android (ML Kit) apps already do this for real.
