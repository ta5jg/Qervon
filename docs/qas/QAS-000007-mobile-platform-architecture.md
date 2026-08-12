<!-- =============================================================================
File:           docs/qas/QAS-000007-mobile-platform-architecture.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  The real, shipped architecture of the native iOS and Android Courier +
  Customer apps.

Specification:
  ADR-000002, ADR-000003, QAS-000005.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000007 — Mobile Platform Architecture

**Status: Implemented.** Four real apps: `mobile/ios/QervonCourierApp`,
`mobile/ios/QervonCustomerApp`, `mobile/android/app-courier`,
`mobile/android/app-customer` — all against the same
`backend/apps/api-gateway` contract.

## Shared shape across both platforms

Both platforms use the same layering, just in each platform's idiom:

| Concern | iOS | Android |
| --- | --- | --- |
| DTOs matching backend JSON | `QervonCore` (SPM) | `core:common` (pure Kotlin/JVM) |
| HTTP client, Bearer + refresh-retry | `QervonNetworking` (SPM) | `core:network` (pure Kotlin/JVM, Retrofit+OkHttp) |
| Token storage, biometric gate | `QervonSecurity` (SPM, Keychain + LocalAuthentication) | `core:security` (EncryptedSharedPreferences + BiometricPrompt) |
| Background courier location | `QervonLocation` (SPM, CoreLocation) | `core:location` (foreground `Service` + FusedLocationProviderClient) |
| Shared theme | `QervonDesignSystem` (SPM) | `core:designsystem` (Compose Material3) |
| Screens | `Features/QervonFeatures` (SPM, one target per screen area) | `feature:*` (one Gradle module per screen area) |

Both refresh-retry implementations follow the same rule: on a single 401,
attempt exactly one token refresh and retry the original request once;
never loop.

## Courier app feature set (both platforms)

Login (password or phone/OTP) with biometric re-lock → online/offline
toggle → job offer with a countdown accept/reject
(`GET /v1/courier/me/offer`, 45s server-side TTL) → active job list with
external navigation (`geo:`/`google.navigation:` deep links, no in-app
map, no Maps API key) → proof-of-delivery (QR/barcode scan, digital
signature, camera photo) → earnings/wallet → profile.

## Customer app feature set (both platforms)

Register/login → address book (map-based picker + reverse geocoding) →
new order with a live, non-binding fare quote
(`GET /v1/customer/fare-quote`) → order history → order detail with live
tracking map + ETA polling, cancel, rate, open a support ticket → profile.

## Platform-specific choices, both deliberate and documented

- **In-app maps:** iOS uses MapKit (a first-party framework, no API key
  needed for the Apple ecosystem); Android uses `osmdroid`
  (OpenStreetMap) specifically because the Google Maps SDK needs a
  billing-enabled API key this environment does not have.
- **QR/barcode scanning:** iOS uses VisionKit's
  `DataScannerViewController` (falls back to a manual toggle on the
  Simulator, which has no camera); Android uses Google ML Kit Barcode
  Scanning + CameraX (works on the emulator's virtual camera too).
- **Background location service:** iOS relies on `CLLocationManager`'s
  background modes; Android uses an explicit foreground `Service`
  (Android 14+ `FOREGROUND_SERVICE_LOCATION` type) because Android's
  background-execution limits are stricter than iOS's for this use case.
- **DI:** Android uses Hilt throughout (many modules, real benefit from
  constructor injection); iOS uses a simpler manual composition root
  (`AppSession`) since SwiftUI's environment-object pattern makes a full
  DI framework less necessary at this app's size.

## What is not implemented on either platform

- **Native push (FCM/APNs sending).** iOS captures a real APNs device
  token and posts it to `POST /v1/push/devices`, but no APNs credential
  exists in this environment to actually send anything. Android does not
  even integrate the FCM SDK, because a missing `google-services.json`
  fails the *build*, not just the runtime, unlike iOS's silent failure.
- **No pickup-evidence capture** — only delivery has QR/signature/photo
  fields on the backend; pickup is a single confirmation tap on both
  platforms.

## References

- ADR-000002 (Kotlin/Android), ADR-000003 (Swift/iOS), QAS-000005 (the
  API contract both platforms consume).
- [mobile/ios/README.md](../../mobile/ios/README.md),
  [mobile/android/README.md](../../mobile/android/README.md).

---

# Revision History

| Version | Date | Description |
| --------- | ------ | ------------- |
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real, shipped architecture of all four apps. |
| 0.3.0 | 2026-08-13 | Removed the "no re-offer cascade" note — that gap is now closed backend-side (see QLS-000003), with no client change needed. |
