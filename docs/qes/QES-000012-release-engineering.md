<!-- =============================================================================
File:           docs/qes/QES-000012-release-engineering.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  How a release actually goes out, per component — backend binaries and
  mobile app builds.

Specification:
  QAS-000014, QMI-000002.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000012 — Release Engineering

## Backend

`scripts/build-release.sh` builds release binaries for the API,
migration runner, and worker; `docs/operations/deployment-runbook.md`
covers the full VPS deploy procedure including the `.previous`-binary
rollback mechanism (see QAS-000014). There is no semantic API version in
the URL path yet (`/v1/...` has never needed a `/v2/...` sibling) — see
QMI-000002 for the policy on when that would change.

## Mobile

- **Android:** `./gradlew :app-courier:assembleDebug
  :app-customer:assembleDebug` produces installable, unsigned debug
  APKs. There is no release-signing configuration or Play Store
  publishing pipeline yet — this phase's scope was "real, installable
  debug builds", not store distribution.
- **iOS:** `mobile/ios/scripts/build-simulator.sh` produces Simulator
  builds (no code signing needed). There is no App Store Connect/
  TestFlight pipeline yet — same reasoning as Android above; this needs
  an Apple Developer Program membership this environment does not have.

## What release notes/changelogs look like today

There is no formal `CHANGELOG.md` — the closest equivalent is each
governance document's own `# Revision History` table (QES-000009) plus
git commit history (QES-000007). A dedicated changelog is a reasonable
future addition once releases are cut on a defined cadence.

## References

- [QAS-000014](../qas/QAS-000014-deployment-architecture.md) (deployment architecture/topology), [QMI-000002](../qmi/QMI-000002-versioning-policy.md) (versioning
  policy), [docs/operations/deployment-runbook.md](../operations/deployment-runbook.md),
  [docs/operations/mobile-release-runbook.md](../operations/mobile-release-runbook.md).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real backend/mobile release process and the current absence of store publishing/signing. |
