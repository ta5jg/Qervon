<!-- =============================================================================
File:           docs/qes/QES-000006-testing-standard.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  How tests are actually organized and what CI actually runs, per
  platform.

Specification:
  QES-000002, QES-000003, QES-000004, QES-000010.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000006 — Testing Standard

## Backend (real, CI-enforced)

- `cargo test --workspace --all-targets` on every push/PR touching
  `backend/**` (`.github/workflows/backend.yml`) — unit tests beside the
  code, plus full-HTTP-flow integration tests in
  `backend/apps/api-gateway/tests/api_flow.rs` that exercise real
  multi-step scenarios (register → login → create order → assign →
  deliver → wallet credited) against the in-memory backend.
- `make test-postgres` (not wired into the current CI workflow — a real
  gap) runs the same integration suite against a real PostgreSQL
  instance, to catch behavior that only diverges on the SQL backend.

## Mobile (real, and CI-enforced as of 2026-08-13)

- **Android:** JVM unit tests in `core:common`/`core:network`
  (`JsonCodingTest`, `RequestEncodingTest`) run via
  `./gradlew :core:common:test :core:network:test`, verifying DTO
  encode/decode against realistic backend JSON — fast, no emulator. Run
  by `.github/workflows/android.yml` on every push/PR touching
  `mobile/android/**`, followed by a real `assembleDebug` build of both
  apps.
- **iOS:** XCTest targets (`QervonCoreTests`, `QervonNetworkingTests`)
  cover the same kind of JSON-shape verification, run via
  `swift test --package-path Packages/QervonKit`. Run by
  `.github/workflows/ios.yml` on every push/PR touching `mobile/ios/**`,
  followed by a real Simulator-SDK build of both apps via
  `scripts/build-simulator.sh`.

## Web (none)

The vanilla HTML/JS pages under `backend/apps/api-gateway/static/` have
no automated test coverage — verification during the 2026-08-12 audit
was done with `curl` against a locally-running server, manually, not via
a repeatable test suite. Adding one (e.g. Playwright driving the real
pages against a locally-started `api-gateway`) is a reasonable future
improvement, not yet done.

## What "done" means for a new feature

A change is not complete until: the backend behavior it depends on has a
passing integration test in `api_flow.rs` covering the happy path *and*
at least one rejection path (wrong tenant, wrong role, invalid state
transition) — this is the pattern every existing test in that file
follows and new tests should match.

## References

- [QES-000002](QES-000002-rust-engineering-standard.md) (Rust testing conventions), [QES-000010](QES-000010-ci-cd-standard.md) (CI/CD standard —
  which workflows are real vs. placeholder).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real per-platform test coverage and the gap between "tests exist" and "CI runs them" for mobile. |
