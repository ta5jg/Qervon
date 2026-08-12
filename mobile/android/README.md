<!-- =============================================================================
File:           mobile/android/README.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-12
Version:        0.1.0

Description:
  Orientation and build instructions for the native Qervon Android apps:
  Courier and Customer.

Specification:
  QAS-000004, QAS-000005, QAS-000007, QES-000003, QES-000004.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# Qervon — Native Android Apps

Two real, Gradle-compilable Kotlin/Jetpack Compose apps sharing a
multi-module project and most of their infrastructure, both backed by the
`backend/apps/api-gateway` HTTP API — the direct Android counterpart of
`mobile/ios/` (Faz-2.2/Faz-2.3), built against the exact same backend
contract.

- **`app-courier`** (`com.qervon.android.courier`) — for couriers.
- **`app-customer`** (`com.qervon.android.customer`) — for customers.

## Structure

```text
mobile/android/
├── settings.gradle.kts            # module registry
├── build.gradle.kts               # root plugin declarations (version catalog aliases)
├── gradle/libs.versions.toml      # centralized dependency versions
├── gradlew, gradlew.bat, gradle/wrapper/   # committed Gradle wrapper (Gradle 8.9)
├── app-courier/                   # Courier app module: Application, MainActivity, NavHost, tabs
├── app-customer/                  # Customer app module — same shell shape as above
├── core/
│   ├── common/                    # pure-Kotlin/JVM: DTOs matching backend JSON, ApiError, formatting
│   ├── network/                   # pure-Kotlin/JVM: Retrofit+OkHttp client, Bearer auth, refresh-and-retry
│   ├── security/                  # Android: EncryptedSharedPreferences token store, BiometricPrompt
│   ├── location/                  # Android: FusedLocationProviderClient + foreground Service (Courier only)
│   └── designsystem/              # Android: shared Compose Material3 theme
└── feature/
    ├── auth/                      # shared: login (password + phone/OTP), biometric lock, registration
    ├── dispatch/                  # Courier: online/offline, job offer with countdown accept/reject
    ├── orders/                    # Courier: active job list, external navigation Intent, pickup
    ├── proof/                     # Courier: ML Kit QR/barcode scan, signature pad, CameraX photo, delivery
    ├── earnings/                  # Courier: wallet balance, period totals, ratings
    ├── profile/                   # Courier: account, phone linking, biometric toggle, logout
    ├── addressbook/                # Customer: saved addresses + osmdroid map picker/Geocoder
    ├── customerorder/             # Customer: new order (live fare quote), history, tracking/ETA/cancel/rate
    └── customerprofile/           # Customer: account, address book, support tickets, notifications, logout
```

`core:common` and `core:network` are plain Kotlin/JVM modules (Retrofit,
OkHttp, and kotlinx.serialization have no Android dependency), which keeps
them fast to build and unit-testable without an emulator/device. Every
other module is an Android library. `feature:auth` is the only feature
module shared by both apps; `feature:addressbook` is shared between
`feature:customerorder` and `feature:customerprofile` (both need the
osmdroid map picker); everything else is app-specific.

## Building

No Android Studio required — everything here was built and verified via
the Gradle CLI in an environment without Homebrew/sudo access.

### Toolchain (already set up in this checkout)

1. **Portable JDK 17** (modern Gradle/AGP require it; the environment had
   JDK 8): downloaded a precompiled Eclipse Temurin 17 macOS/aarch64
   build and extracted it to `~/.local/qervon-tools/jdk-17/` — the same
   "no package manager, download a precompiled binary" pattern used for
   XcodeGen in `mobile/ios/README.md`.
2. **Android SDK**: reused the already-installed `~/Library/Android/sdk`
   (`ANDROID_HOME`); `local.properties` (gitignored, machine-specific)
   points `sdk.dir` at it.
3. **Gradle 8.9**: downloaded the distribution and ran `gradle wrapper`
   once to generate the **real, committed** `gradlew`/`gradlew.bat`/
   `gradle/wrapper/gradle-wrapper.jar` — standard Android practice, unlike
   `.gradle/`/`build/` which stay gitignored.

To build on a fresh machine, only a JDK 17 and the Android SDK
command-line tools are required; export `JAVA_HOME`/`ANDROID_HOME` (or set
`sdk.dir` in `local.properties`) and run:

```bash
cd mobile/android
./gradlew :app-courier:assembleDebug :app-customer:assembleDebug
```

This produces real, installable, unsigned debug APKs:

```text
app-courier/build/outputs/apk/debug/app-courier-debug.apk
app-customer/build/outputs/apk/debug/app-customer-debug.apk
```

Install directly via `adb install <path>`, or open the project root in
Android Studio and run the `app-courier`/`app-customer` run configurations.

## Backend

Defaults to `http://10.0.2.2:8080` in both apps' `ApiEnvironment.kt`
(`10.0.2.2` is the Android Emulator's alias for the host machine's
`localhost`). Run the backend locally first — see the root
[README.md](../../README.md) and [backend/README.md](../../backend/README.md).
On a **real device**, `10.0.2.2` does not resolve; change `BASE_URL` in
`app-courier/.../ApiEnvironment.kt` / `app-customer/.../ApiEnvironment.kt`
to your machine's LAN IP and rebuild. Unlike the iOS apps, there is no
in-app "Sunucu Adresi" settings field yet — this is a real, documented gap
relative to the iOS client, not a hidden one.

## Architectural decisions (with honesty notes)

- **External navigation (Courier)**: same pattern as iOS's external-nav-app
  picker — a `google.navigation:`/`geo:` `Intent`, no in-app map, no Maps
  API key.
- **In-app map (Customer live tracking, address picker)**: **osmdroid**
  (OpenStreetMap-based, free, no API key) — the Google Maps SDK requires a
  billing-enabled API key/credential that does not exist in this
  environment, so it is not used.
- **Address search/geocoding**: Android's built-in `android.location.Geocoder`
  (on-device, no extra credential) + tap-to-drop-pin on the osmdroid map.
- **QR/barcode scanning (delivery)**: Google ML Kit Barcode Scanning
  (on-device, no API key) + CameraX — the real Android counterpart of the
  iOS client's VisionKit `DataScannerViewController` screen.
- **Native push (FCM)**: **not integrated in this phase.** Firebase Cloud
  Messaging requires a `google-services.json` (Firebase project
  credential) at build time, and without it the
  `com.google.gms.google-services` Gradle plugin **fails the build**
  outright — unlike iOS, where a missing Push entitlement merely fails
  silently at runtime without breaking compilation. Since no such
  credential exists in this environment, omitting the SDK entirely was the
  only honest option; see `BACKEND_BACKLOG.md` for the boundary note.
  `POST /v1/push/devices` is unchanged server-side — only the Android
  client doesn't call it yet.
- **DI**: Hilt, used throughout for ViewModel constructor injection across
  the many feature modules — more mechanical benefit here than iOS's
  simpler composition-root `AppSession`, given Android's Activity/ViewModel
  lifecycle.
- **Background location (Courier)**: a real foreground `Service`
  (`core:location`'s `CourierLocationService`, Android 14+
  `FOREGROUND_SERVICE_LOCATION` type) started/stopped when the courier
  toggles online/offline — not WorkManager, which is not designed for
  continuous high-frequency updates.
- **Local storage**: no Room — the scope is thin enough that
  `EncryptedSharedPreferences` (tokens) and plain `SharedPreferences`
  (settings) suffice. A delivery photo is saved to the app's private
  `filesDir` and never uploaded (see honesty notes below).

## Honesty notes (what is real vs. intentionally deferred)

### Both apps

- **Biometric unlock is local-only**: it gates access to the encrypted
  token store already on the device via `BiometricPrompt`. There is no
  backend biometric API.
- **No native push** — see the FCM note above. This is a real, larger gap
  than iOS's (which at least captures a real, if unusable, APNs token).

### Courier app

- **Pickup has no proof-of-delivery capture**, matching the backend: `POST
  /v1/courier/orders/{id}/pickup` takes no body. The Pickup action is a
  single honest confirmation tap, not a fake QR/photo capture with no
  effect.
- **Delivery evidence**: QR/barcode scanning is real, on-device ML Kit
  Barcode Scanning over a live CameraX preview. The signature pad is a
  real Compose `Canvas` capturing finger-drawn strokes, exported as a real
  `digital_signature_base64` PNG. The camera photo capture (CameraX
  `ImageCapture`) is real, but is saved **locally only** (the app's private
  `filesDir/delivery_photos/`) and is never sent as `photo_evidence_url` —
  the backend expects an already-hosted URL and has no image-upload
  endpoint yet, so sending a fabricated or local-only path would
  misrepresent that field. This mirrors the iOS client's identical gap.
- **Earnings never show total distance** — the backend has no per-courier
  distance aggregation, so no such number is fabricated. Period earnings
  (today/week/month) are computed client-side from the wallet's
  transaction history, since the backend has no aggregation endpoint for
  that either.

### Customer app

- **Fares are real and server-computed, never client-supplied.** The
  "Yeni Sipariş" screen shows a live, non-binding quote from
  `GET /v1/customer/fare-quote`, but `POST /v1/customer/orders` always
  recomputes the authoritative fare server-side from the same
  pickup/dropoff — the client can never manipulate what it is charged.
- **Live tracking is polling-based, not push/WebSocket**: the order detail
  screen polls `GET /v1/orders/{id}/tracking` and
  `GET /v1/customer/orders/{id}/eta` every ~5 seconds while the order is
  `courier_assigned`/`in_transit`. ETA has no real traffic data.
- **Address search/pin-drop is real, on-device** (osmdroid +
  `android.location.Geocoder` reverse geocoding) — no backend involvement,
  no fabricated coordinates.
- **Registration never returns tokens** (matching the backend's actual
  contract): `POST /v1/auth/register` only creates the account; the app
  immediately follows up by navigating back to a real password login.

## Not yet built (later phases)

- The in-app "server address" setting the iOS apps have (currently a
  build-time constant here).
- Real device / Play Store signing and release builds (this phase only
  targets unsigned, installable debug APKs).
- Native push (FCM) — needs a real Firebase project credential.
- Tenant pricing admin UI (the `/v1/pricing` API exists; no panel screen).
