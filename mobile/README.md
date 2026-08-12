<!-- =============================================================================
File:           mobile/README.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Orientation for the mobile/ area: two real, native, per-platform app
  suites, both built against the same backend/apps/api-gateway contract.

Specification:
  QAS-000007, ADR-000002, ADR-000003.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# Mobile — Native iOS + Android

```text
mobile/
├── ios/       Native Swift/SwiftUI apps (Courier + Customer), see ADR-000003
└── android/   Native Kotlin/Jetpack Compose apps (Courier + Customer), see ADR-000002
```

Both suites implement the same PDF-vision scope — courier dispatch/proof-of-
delivery/earnings, customer ordering/tracking/support — against the exact
same `backend/apps/api-gateway` HTTP contract. See
[mobile/ios/README.md](ios/README.md) and
[mobile/android/README.md](android/README.md) for build instructions and
per-platform honesty notes (what's real vs. deliberately deferred).

An earlier Flutter prototype (`mobile/courier_app/`, `mobile/customer_app/`)
existed at one point but had no `pubspec.yaml` and could not build; it was
removed once native iOS and Android replaced it (see ADR-000002).

## References

- QAS-000007 (mobile platform architecture).
- ADR-000002 (Kotlin for Android), ADR-000003 (Swift for iOS).

---

## Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten to describe the real native iOS + Android apps; noted removal of the dead Flutter prototype. |
