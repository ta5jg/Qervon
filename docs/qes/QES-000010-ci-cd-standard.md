<!-- =============================================================================
File:           docs/qes/QES-000010-ci-cd-standard.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  Honest inventory of .github/workflows: which pipelines are real, which
  are still empty placeholders, and (as of 2026-08-13) the fake
  mobile-build.yml step that was found and replaced with real per-platform
  CI.

Specification:
  QES-000002 through QES-000006.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000010 — CI/CD Standard

**Status: Implemented (backend + both mobile platforms + backend
dependency scanning) — a few workflow files remain empty placeholders.**

## Real, working pipelines

| Workflow | What it actually does |
| --- | --- |
| `backend.yml` | `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test --workspace --all-targets` on every push/PR touching `backend/**`. |
| `android.yml` | `./gradlew :core:common:test :core:network:test` (JVM unit tests) then `./gradlew :app-courier:assembleDebug :app-customer:assembleDebug`, uploading both real debug APKs as build artifacts. Fails on any real Gradle build or test regression. |
| `ios.yml` | `swift test --package-path Packages/QervonKit` (real XCTest suite) then `./scripts/build-simulator.sh`, which builds every Swift package library scheme and both app targets for the Simulator SDK. Fails on any real Swift compile or test regression. |
| `security.yml` | `rustsec/audit-check` against the backend's `Cargo.lock`, on every push/PR touching `backend/**` and weekly on a schedule (so a newly-disclosed advisory is caught even with no code change). |
| `deploy.yml` | Deploys `docs/` to GitHub Pages as a static site. Unrelated to product deployment (see QAS-000014 for the real VPS/systemd deploy process). |

## Fixed: the fake `mobile-build.yml` (2026-08-13)

`mobile-build.yml` used to exist with a `build-android` job that only ran
`echo "... verified successfully."` (no Gradle command at all) and a
`build-ios` job that ran a syntax-only `xcrun swiftc -parse` against
stale, pre-restructuring file paths — neither could ever fail on a real
regression. It has been **deleted** and replaced by the real
`android.yml`/`ios.yml` workflows above, each independently verified by
actually running its commands locally before being committed.

## Empty placeholders (no jobs at all)

`architecture.yml`, `release.yml`, `web.yml` still contain only the file
header comment — no `on:`/`jobs:` keys. Each name implies real intended
scope (a governance/architecture-doc linter that could, for example,
fail if a placeholder PDF-dump pattern like the one this whole
documentation series used to contain ever reappears; a tagged-release
binary-attachment workflow; a web-pages accessibility/lint check) that
has not been built.

`.github/dependabot.yml` is likewise an empty placeholder — no
`updates:` configuration — so **no automated dependency-update PRs are
actually being generated today** despite the file's presence suggesting
otherwise. See QES-000011.

## Recommended next steps (not yet implemented)

1. Fill in `architecture.yml` with a real governance-doc lint (e.g. fail
   if any `docs/{qmi,qfs,qas,qes,qls,adr}/*.md` file is missing a
   `**Status:**` line, or if the PDF-dump placeholder string ever
   reappears).
2. Fill in `dependabot.yml` with a real `updates:` block, or remove the
   file if dependency updates will be handled manually.
3. Extend dependency scanning to Gradle/SPM (no equivalent of
   `rustsec/audit-check` is wired in for either mobile platform yet).

## References

- [QES-000002](QES-000002-rust-engineering-standard.md) through [QES-000006](QES-000006-testing-standard.md) (what each workflow enforces),
  [QES-000001](QES-000001-engineering-principles.md) (the honesty principle this document follows in recording
  both the fix and what's still missing).

---

# Revision History

| Version | Date | Description |
| --------- | ------ | ------------- |
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with an honest, file-by-file audit of every workflow's real vs. placeholder vs. fake status. |
| 0.3.0 | 2026-08-13 | Replaced the fake `mobile-build.yml` with real, independently-verified `android.yml`/`ios.yml`; filled in `security.yml` with `cargo audit`. |
