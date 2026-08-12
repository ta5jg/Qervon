<!-- =============================================================================
File:           mobile/ios/README.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  Orientation and build instructions for the native Qervon iOS apps:
  Courier (Faz-2.2) and Customer (Faz-2.3).

Specification:
  QAS-000004, QAS-000005, QAS-000007, QES-000003, QES-000004.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# Qervon — Native iOS Apps

Two real, compilable SwiftUI apps sharing one Xcode project and most of
their infrastructure, both backed by the `backend/apps/api-gateway` HTTP
API:

- **`QervonCourierApp`** (`com.qervon.ios.courier`, Faz-2.2) — for couriers.
- **`QervonCustomerApp`** (`com.qervon.ios.customer`, Faz-2.3) — for customers.

## Structure

```text
mobile/ios/
├── Project.yml                    # XcodeGen source of truth (see below)
├── QervonCourierApp.xcodeproj/    # generated — do not edit, do not commit
├── QervonCourierApp/              # Courier app target: entry point, session, app shell
│   ├── QervonCourierApp.swift     # @main
│   ├── AppSession.swift           # composition root (Keychain + HTTP client + API)
│   ├── AppDelegate.swift          # APNs device token capture (see honesty note below)
│   ├── RootView.swift             # switches Auth / Biometric-lock / Main tabs
│   ├── MainTabView.swift          # Ana Sayfa / Kazanç / Profil tabs
│   └── Assets.xcassets/
├── QervonCustomerApp/             # Customer app target — same shell shape as above
│   ├── QervonCustomerApp.swift
│   ├── AppSession.swift
│   ├── AppDelegate.swift
│   ├── RootView.swift             # includes registration, not just login
│   ├── MainTabView.swift          # Sipariş Ver / Siparişlerim / Profil tabs
│   └── Assets.xcassets/
├── Packages/QervonKit/            # cross-cutting infrastructure (one SPM package,
│   │                               multiple library products/targets), shared by both apps:
│   ├── QervonCore                 #   shared DTOs matching backend JSON, APIError, date/money formatting
│   ├── QervonNetworking           #   async/await HTTP client, Bearer auth, one-shot refresh-and-retry
│   ├── QervonSecurity             #   Keychain token store, biometric gate, local preferences
│   ├── QervonLocation             #   CoreLocation broadcaster -> POST /v1/courier/me/location (Courier only)
│   └── QervonDesignSystem         #   shared theme, buttons, text fields
└── Features/QervonFeatures/       # screens (one SPM package, multiple targets):
    ├── AuthFeature                #   shared: login (password + phone/OTP), biometric lock, registration
    ├── DispatchFeature            #   Courier: online/offline, job offer with countdown accept/reject
    ├── OrdersFeature              #   Courier: active job detail (pickup/deliver/navigate)
    ├── MapsFeature                #   Courier: external navigation app picker (deep links only)
    ├── ProofOfDeliveryFeature     #   Courier: pickup confirmation, delivery evidence capture
    ├── EarningsFeature            #   Courier: wallet balance, period totals, ratings
    ├── ProfileFeature             #   Courier: account, phone linking, biometric toggle, logout
    ├── AddressBookFeature         #   Customer: saved addresses + MapKit search/pin picker
    ├── CustomerOrderFeature       #   Customer: new order (live fare quote), history, tracking/ETA/cancel/rate
    └── CustomerProfileFeature     #   Customer: account, address book, support tickets, notifications, logout
```

`Packages/QervonKit` and `Features/QervonFeatures` each ship as a single SPM
package with several library targets (rather than one package per module) to
cut down on repetitive `Package.swift` boilerplate — this is a deliberate
simplification and does not change the dependency shape between modules.
`AuthFeature` is the only feature target shared by both apps; everything
else is app-specific.

## Building

**`QervonCourierApp.xcodeproj` is generated and is not committed** (the
project name is kept for historical continuity — it now contains both app
targets). [Project.yml](Project.yml) is the only source of truth.

1. Install [XcodeGen](https://github.com/yonaskolb/XcodeGen) (a precompiled
   release binary works fine — no Homebrew required):

   ```bash
   gh release download 2.46.0 --repo yonaskolb/XcodeGen --pattern xcodegen.zip -D /tmp/xcodegen-download
   unzip -o /tmp/xcodegen-download/xcodegen.zip -d /tmp/xcodegen-download
   # copy /tmp/xcodegen-download/xcodegen somewhere on your PATH, e.g. ~/.local/qervon-tools/xcodegen
   ```

2. Generate the project:

   ```bash
   cd mobile/ios
   xcodegen generate
   ```

3. Open `QervonCourierApp.xcodeproj` in Xcode and run the `QervonCourierApp`
   or `QervonCustomerApp` scheme, **or** build both headlessly from the
   terminal:

   ```bash
   ./scripts/build-simulator.sh
   ```

### Why there's a build script instead of a one-line `xcodebuild`

`xcodebuild build -scheme QervonCourierApp -sdk iphonesimulator` refuses to
run unless a Simulator *runtime* is installed (Xcode > Settings > Platforms),
because resolving a destination for an "application" scheme requires a
concrete simulator device. Compiling Swift code, however, only needs the
Simulator *SDK*, which ships with Xcode itself — no runtime required.
[`scripts/build-simulator.sh`](scripts/build-simulator.sh) builds every local
Swift package product via its library scheme (no destination needed) and
then builds each app target directly, pointing `SYMROOT`/`OBJROOT` at the
same DerivedData the package builds used. This was verified end-to-end in
this environment (no Simulator runtime installed) and produced real
`QervonCourierApp.app` and `QervonCustomerApp.app` arm64 Mach-O binaries. If
a Simulator runtime *is* installed on your machine, the normal Xcode "Run"
workflow works too and additionally lets you boot the app in a simulator.

## Backend

Defaults to `http://127.0.0.1:8080` (the Simulator's view of the host Mac's
loopback address). Run the backend locally first — see the root
[README.md](../../README.md) and [backend/README.md](../../backend/README.md).
On a real device, `127.0.0.1` doesn't resolve to your Mac; set your Mac's LAN
IP from the Profile screen's "Sunucu Adresi" field (present in both apps).

## Honesty notes (what is real vs. intentionally deferred)

### Both apps

- **Biometric unlock is local-only**: it gates access to the Keychain
  tokens already on the device. There is no backend biometric API.
- **Native push**: `AppDelegate` captures a real APNs device token if iOS
  ever grants one and forwards it to `POST /v1/push/devices`. In this
  environment (Simulator, no Apple Developer Push entitlement) that will
  never happen, and the app fails silently rather than fabricating a token.
  No APNs/FCM sending is wired server-side either — see `BACKEND_BACKLOG.md`.

### Courier app

- **Pickup has no proof-of-delivery capture** because
  `POST /v1/courier/orders/{id}/pickup` takes no body server-side — the
  backend only models delivery evidence, not pickup evidence. The Pickup
  screen is therefore a single honest confirmation step, not a fake
  QR/photo capture with no effect.
- **Delivery evidence**: QR/barcode scanning uses `VisionKit`'s
  `DataScannerViewController` for real (iOS 16+, works on a physical
  device; `DataScannerViewController.isSupported` is `false` on the
  Simulator, so a manual "doğrulandı" toggle is offered as an honest
  fallback — never a fake scan result). The signature pad is fully real
  end-to-end (`digital_signature_base64` is a real backend field). The
  camera photo capture is real, but is stored **locally only**
  (`QervonCourierApp`'s Documents directory) and is *not* sent as
  `photo_evidence_url` — the backend expects an already-hosted URL and has
  no image upload endpoint yet, so sending a fabricated or local-only path
  would misrepresent that field.
- **Statistics** never show total distance — the backend has no per-courier
  distance aggregation, so no such number is fabricated. Period earnings
  (today/week/month) are computed client-side from the wallet's transaction
  history, since the backend has no aggregation endpoint for that either.

### Customer app

- **Fares are real and server-computed, never client-supplied.** The "Yeni
  Sipariş" screen shows a live, non-binding quote from
  `GET /v1/customer/fare-quote` (distance-based, via
  `qervon_domain::DeliveryPricing`), but `POST /v1/customer/orders` always
  recomputes the authoritative fare server-side from the same pickup/dropoff
  — a client can never manipulate what it is charged. Tenants can configure
  their own pricing via `GET`/`PUT /v1/pricing` (API-only in this phase, no
  admin panel UI yet); an unconfigured tenant gets a real, documented
  default (₺10 base + ₺2.50/km, ₺15 minimum).
- **Live tracking is polling-based, not push/WebSocket**, consistent with
  the Courier app's job-offer polling pattern: the order detail screen polls
  `GET /v1/orders/{id}/tracking` and `GET /v1/customer/orders/{id}/eta`
  every ~5 seconds while the order is `courier_assigned`/`in_transit`. ETA
  has no real traffic data — it reuses the same distance/vehicle-type
  estimate the AI Dispatcher uses internally.
- **"Favoriler" = the saved address book.** There is no separate
  favorite-courier or favorite-order concept in the backend.
- **No order-creation photo attachment.** The backend has no such field on
  `CreateCustomerOrderRequest`; rather than fabricate one, the field is
  simply not offered.
- **Address search/pin-drop is real, on-device MapKit/CoreLocation**
  (`MKLocalSearch` + `CLGeocoder` reverse geocoding) — no backend
  involvement, and no fabricated coordinates.
- **Registration never returns tokens** (matching the backend's actual
  contract): `POST /v1/auth/register` only creates the account and tenant
  membership; the app immediately follows up with a real password login
  using the same credentials.

## Not yet built (later phases)

- Android apps (Courier + Customer, Kotlin/Compose) — still prototypes.
- Real device / App Store code signing (requires an Apple Developer team;
  this phase only targets Simulator-buildable, automatically-signed builds).
- Multi-step automatic re-offer cascade when a courier rejects/times out —
  today the order simply returns to `Pending` for an operator to reassign.
- Tenant pricing admin UI (the `/v1/pricing` API exists; no panel screen yet).
