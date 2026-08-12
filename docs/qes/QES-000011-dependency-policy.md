<!-- =============================================================================
File:           docs/qes/QES-000011-dependency-policy.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  Rules for adding new dependencies and how versions are pinned, across
  Cargo, Gradle, SPM, and CDN scripts.

Specification:
  QES-000002, QES-000003, QES-000004, QES-000010.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000011 — Dependency Policy

## Version pinning

- **Rust** (`backend/Cargo.toml`): workspace-level `[workspace.dependencies]`
  so every crate uses the same version of a shared dependency; `Cargo.lock`
  is committed.
- **Kotlin/Gradle** (`mobile/android/gradle/libs.versions.toml`): a
  single version catalog, no per-module hardcoded version strings — a new
  dependency is added to the catalog once and referenced by alias
  everywhere it's needed.
- **Swift/SPM**: exact or range versions in each `Package.swift`;
  `Package.resolved` is committed.
- **CDN scripts** (the shipped web pages, see QAS-000008): always an
  exact version in the URL (`lucide@1.31.0`, `leaflet@1.9.4`), never
  `@latest` — an unpinned CDN script is a supply-chain risk found and
  fixed during the 2026-08-12 web audit.

## Adding a new dependency

Before adding one, check: is there already a dependency in this
workspace doing something close enough? (e.g. don't add a second HTTP
client to the Rust backend when `reqwest`/direct `axum` usage already
covers the need). Prefer a well-maintained crate/library with a real
release history over a marginally-more-elegant one with few users — this
is production infrastructure, not a demo.

## Automated update tracking

**Not currently implemented.** `.github/dependabot.yml` exists but is an
empty placeholder with no `updates:` configuration — despite its
presence, no automated dependency-update PRs are being generated (see
QES-000010). Until this is filled in, dependency updates are a manual,
periodic task with no tooling reminder.

## Vulnerability scanning

**Implemented for the backend, not for mobile.** `.github/workflows/security.yml`
runs `rustsec/audit-check` against `backend/Cargo.lock` on every relevant
push/PR and weekly on a schedule (see QES-000010). No equivalent
Gradle/SPM dependency-vulnerability scan exists yet for either mobile
platform.

`backend/.cargo/audit.toml` ignores exactly one advisory,
**RUSTSEC-2023-0071** (a timing side-channel in the `rsa` crate, pulled in
transitively via `web-push -> jwt-simple -> rsa` for `apps/worker`'s real
VAPID Web Push sender). It has no fixed upgrade from its maintainers as of
this writing, and the exposed surface — `jwt-simple`'s optional RSA JWT
support — is not code path Qervon uses (Qervon's own tokens are a custom
HMAC scheme; see `backend/apps/api-gateway/src/auth.rs`). This is the
*only* advisory ignored; every other advisory found by `cargo audit` must
fail CI, and this entry must be revisited if `jwt-simple`/`web-push` ship
a fix.

## References

- [QES-000002](QES-000002-rust-engineering-standard.md) through [QES-000004](QES-000004-swift-engineering-standard.md) (per-language dependency conventions),
  [QES-000010](QES-000010-ci-cd-standard.md) (the CI gaps this document's "not implemented" sections
  point back to).

---

# Revision History

| Version | Date | Description |
| --------- | ------ | ------------- |
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real per-ecosystem pinning conventions and the honest gap in automated scanning. |
| 0.3.0 | 2026-08-13 | `cargo audit` is now real and CI-enforced (see QES-000010); documented the one ignored advisory (RUSTSEC-2023-0071) and why. |
