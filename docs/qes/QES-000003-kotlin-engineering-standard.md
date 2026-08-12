<!-- =============================================================================
File:           docs/qes/QES-000003-kotlin-engineering-standard.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Kotlin/Jetpack Compose conventions used across mobile/android's
  multi-module Gradle project.

Specification:
  ADR-000002, QAS-000007.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000003 — Kotlin Engineering Standard

## Module boundaries

Follow `mobile/android`'s established shape (see QAS-000007): pure-JVM
modules (`core:common`, `core:network`) for anything with no genuine
Android dependency, Android-library modules for anything that does, one
`feature:*` module per screen area. A new screen area gets its own
Gradle module rather than being added to an existing one, even if small
— this keeps build graphs (and incremental build times) from degrading.

## State management

- One `@HiltViewModel` per screen, exposing a single `StateFlow<UiState>`
  (a plain data class) plus a `SharedFlow` for one-shot events (e.g.
  "navigate away after login succeeds") — never exposing mutable state
  directly to Compose.
- ViewModels never hold an Android `Context`/`Activity` reference
  directly; anything needing one (biometric prompt, permission checks)
  takes it as a parameter from the Composable at call time.

## Dependency injection

Hilt throughout, `@Provides` methods in a per-app `di/NetworkModule`
binding the shared `core:network`/`core:security` types with that app's
own base URL and token-store namespace (see QAS-000007) — `core:*`
modules stay app-agnostic; only the `app-courier`/`app-customer` modules
know their own configuration.

## Networking

Retrofit + OkHttp + kotlinx.serialization, one `QervonApiService`
interface listing every endpoint, wrapped by a `QervonApi` façade that
converts `Response<T>`/exceptions into a single `QervonApiException`
hierarchy — feature code never touches Retrofit types directly.

## Compose conventions

- `@OptIn(ExperimentalMaterial3Api::class)` at the function level for
  screens using `Scaffold`'s `TopAppBar`, not suppressed globally.
- Shared visual primitives (`QervonCard`, `QervonPrimaryButton`,
  `StatusPill`) live in `core:designsystem`; a screen should not
  hand-roll its own card/button style.
- No screen calls a suspend network function directly from a Composable
  body — always through a ViewModel method, so the call survives
  recomposition/configuration change correctly.

## Testing

JVM unit tests in `core:common`/`core:network` (no Android dependency,
fast) cover JSON encode/decode shape matching the backend contract —
see `JsonCodingTest`, `RequestEncodingTest` as the pattern to follow for
new DTOs.

## References

- [ADR-000002](../adr/ADR-000002-use-kotlin-for-android.md) (why Kotlin/Compose), [QAS-000007](../qas/QAS-000007-mobile-platform-architecture.md) (the real module
  architecture this codifies).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real conventions established while building mobile/android. |
