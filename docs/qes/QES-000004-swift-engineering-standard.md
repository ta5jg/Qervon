<!-- =============================================================================
File:           docs/qes/QES-000004-swift-engineering-standard.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Swift/SwiftUI conventions used across mobile/ios's SPM-package-based
  project.

Specification:
  ADR-000003, QAS-000007.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000004 — Swift Engineering Standard

## Project structure

`Project.yml` (XcodeGen) is the single source of truth for the Xcode
project; the generated `.xcodeproj` is never hand-edited and is
gitignored (see [mobile/ios/README.md](../../mobile/ios/README.md)). New
shared code goes into a library target under `Packages/QervonKit`
(infrastructure) or `Features/QervonFeatures` (screens) rather than
directly into an app target, so it's automatically shared between
`QervonCourierApp` and `QervonCustomerApp` where applicable.

## Concurrency

`async`/`await` throughout the networking layer; no completion-handler
callbacks in new code. `@MainActor` on any type that touches UI state
directly. `CLLocationManagerDelegate` conformance is marked
`@preconcurrency` where the delegate protocol predates Swift's strict
concurrency checking, with a comment explaining why, rather than silently
suppressing the warning.

## Error handling

Networking errors are a single `APIError` enum (mirroring the backend's
`{status, title, detail}` shape from QAS-000005); a `ViewModel` catches
this at the call site and maps it to a user-facing message string — no
raw `Error` is ever shown directly to a user.

## SwiftUI conventions

- One `@Observable`/`ObservableObject` view model per screen, holding
  `@Published` state; views read state, never construct network calls
  directly in a `body`.
- Shared visual primitives (`QervonCard`, `QervonButtonStyle`,
  `QervonTextField`) live in `QervonDesignSystem`; a screen should not
  redefine its own button/card style.
- Platform-conditional code (`#if canImport(UIKit)`) is used sparingly
  and only where a package genuinely needs to build for both iOS and
  macOS (e.g. for `swift build` sanity checks in this environment without
  full Xcode) — see `QervonLocation`/`QervonDesignSystem` for the
  established pattern.

## Security

Tokens live in the Keychain (`QervonSecurity`'s `TokenStore`), never in
`UserDefaults`. Biometric gating uses `LocalAuthentication`, always with
a graceful "not available" path (Simulator/no biometric hardware) rather
than assuming success.

## Testing

XCTest targets per SPM package (`QervonCoreTests`, `QervonNetworkingTests`)
covering JSON decode shape against realistic backend payloads — the same
pattern as Android's JVM tests (QES-000003), so both platforms verify
their DTOs the same way.

## References

- [ADR-000003](../adr/ADR-000003-use-swift-for-ios.md) (why Swift/SwiftUI), [QAS-000007](../qas/QAS-000007-mobile-platform-architecture.md) (the real project
  architecture this codifies).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real conventions established while building mobile/ios. |
