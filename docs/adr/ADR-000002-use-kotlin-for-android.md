<!-- =============================================================================
File:           docs/adr/ADR-000002-use-kotlin-for-android.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Architecture Decision Record: use native Kotlin/Jetpack Compose for the
  Android Courier and Customer apps (not Flutter, not Java).

Specification:
  QMI-000000, QAS-000007, QES-000003.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# ADR-000002 — Use Kotlin/Jetpack Compose for Android

- **Status:** Accepted — implemented.
- **Date:** 2026-08-12 (Faz-2 mobile phase; supersedes an earlier,
  never-implemented Flutter placeholder under `mobile/`).
- **Deciders:** Irfan Gedik.

## Context

The product vision requires two Android apps — Courier and Customer — with
continuous background GPS publishing, biometric-gated login, camera-based
QR/barcode delivery evidence, and a live-tracking map, all against the
same Rust backend the iOS apps use. The repository initially held a
cross-platform Flutter placeholder that was never built out. The choice
was between finishing that Flutter path, or building native Kotlin apps
matching the native Swift/iOS approach already decided (ADR-000003).

## Decision

Build native Android apps in Kotlin with Jetpack Compose, as a
multi-module Gradle project under `mobile/android/`:

- `core:common`, `core:network` — pure-Kotlin/JVM modules (DTOs,
  Retrofit+OkHttp+kotlinx.serialization), no Android dependency.
- `core:security`, `core:location`, `core:designsystem` — Android library
  modules (EncryptedSharedPreferences + BiometricPrompt, FusedLocation +
  foreground Service, Compose Material3 theme).
- `feature:*` — one module per screen area, `feature:auth` shared by both
  apps, the rest app-specific.
- `app-courier`, `app-customer` — the two installable applications, each
  with Hilt DI wiring its own `QervonApi`/token-store instances.

The Flutter placeholder was removed as part of this decision; it held no
real implementation to preserve.

## Consequences

- **Positive:** first-class access to `BiometricPrompt`, CameraX, and
  Google ML Kit Barcode Scanning without a plugin-bridge layer; identical
  architecture shape to the iOS app (see ADR-000003) makes the two mobile
  codebases easy to reason about side by side; `./gradlew assembleDebug`
  produces a real, installable APK with no extra toolchain beyond a JDK
  and the Android SDK.
- **Negative:** UI code is not shared between iOS and Android — each
  platform's screens are written twice, once in SwiftUI and once in
  Compose. This is an accepted cost of going fully native rather than
  cross-platform.
- **Neutral:** Google Maps SDK was deliberately not adopted (no billing
  credential in this environment); `osmdroid` is used instead — see
  QAS-000007 for the mobile platform architecture and its honesty notes.

## Alternatives Considered

- **Flutter** (the original placeholder): a single Dart codebase for both
  platforms, but weaker access to platform-specific biometric/camera APIs
  without third-party plugins, and it diverges from the native-first
  decision already made for iOS.
- **Java**: still viable on Android, but Kotlin's null-safety and
  coroutines materially simplify the async network/location code this app
  is built around.

## References

- [QAS-000007](../qas/QAS-000007-mobile-platform-architecture.md) (mobile platform architecture), [QES-000003](../qes/QES-000003-kotlin-engineering-standard.md) (Kotlin engineering
  standard), [ADR-000003](ADR-000003-use-swift-for-ios.md) (Swift for iOS, the sibling decision).
- [mobile/android/README.md](../../mobile/android/README.md).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the actual decision, module structure, and Flutter-placeholder removal noted. |
