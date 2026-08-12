<!-- =============================================================================
File:           docs/qmi/QMI-000002-versioning-policy.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  How version numbers in document headers and in shipped software are
  assigned and incremented.

Specification:
  QMI-000000, QMI-000001.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QMI-000002 — Versioning Policy

## Document versions (the `Version:` header field)

Every governance document's header `Version:` field follows semantic-ish
versioning applied to *documentation*, not to running software:

- **Patch (0.0.x):** typo/formatting fixes, link corrections, no change
  to the document's factual claims.
- **Minor (0.x.0):** a real content change — new section, corrected
  claim, status change (e.g. Vision → Implemented) — that does not
  invalidate how other documents reference this one.
- **Major (x.0.0):** reserved for a restructuring severe enough that
  cross-references from other documents likely need to be re-checked
  (e.g. splitting one document into two, changing its ID).

The jump from `0.1.0` (auto-generated placeholder) to `0.2.0` across all
74 files on 2026-08-12 is a minor bump by this rule — the factual content
changed completely, but no document was split, renamed, or renumbered.

## Software versions

Software components version independently of the documentation:

- **Backend** (`backend/Cargo.toml` `[workspace.package] version`):
  currently `0.1.0` for every crate — pre-1.0, no stability guarantee on
  internal crate APIs yet, though the *external* HTTP contract in
  `api-contracts` is what mobile/web clients actually depend on and
  should be changed carefully (see QES-000012 for release practices).
- **Mobile apps** (`versionName`/`versionCode` in Android's
  `build.gradle.kts`, the iOS target's marketing version): each app
  versions independently since they ship to independent app stores.
- **API contract:** no separate version number exists yet for the HTTP
  contract itself (e.g. no `/v2/...` prefix has ever been needed). If a
  breaking change to a shipped endpoint becomes necessary, introduce a
  versioned path rather than breaking existing mobile clients in place —
  see QES-000012 (release engineering).

## References

- QMI-000001 (document governance), QES-000012 (release engineering).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the actual versioning rules for both docs and software. |
