<!-- =============================================================================
File:           docs/adr/ADR-000004-use-react-typescript-for-web.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Architecture Decision Record: the originally-planned React/TypeScript
  web platform was never implemented, and the empty scaffold was removed
  in favor of the vanilla HTML/JS pages that already existed and worked.

Specification:
  QMI-000000, QAS-000008, QES-000005.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# ADR-000004 — Web Platform Technology: React/TypeScript (Superseded)

- **Status:** Superseded — the React/TypeScript platform this ADR
  originally proposed was never implemented; the actual decision is
  recorded below.
- **Date:** original intent 2026-08-05; superseded 2026-08-12.
- **Deciders:** Irfan Gedik.

## Original Context and Intent

The source architecture PDFs describe an ambitious web platform (admin
dashboard, corporate customer portal) as a React + TypeScript + Vite +
Tailwind + TanStack Query + Zustand single-page application, matching
QAS-000008's original scope. A `web/` directory was scaffolded with this
structure in mind: `apps/platform`, a dozen `features/*` folders, and
`packages/{api-client,auth,design-system,...}`.

## What Actually Happened

The `web/` scaffold was never filled in. Every file under it — 43 files —
contained only a repeated license-header comment; there was no
`package.json` with real dependencies, no lockfile, no `node_modules`, and
no component/route/API code. It could not install or build.

Meanwhile, a **separate**, genuinely working web surface already existed
and had existed from early in the project: vanilla HTML/CSS/JS pages under
`backend/apps/api-gateway/static/`, served directly by Axum via
`include_str!` (`/`, `/customer.html`, `/mobile-customer.html`,
`/mobile-courier.html`, `/login`, `/setup`). These pages call the real
`/v1/...` API, use `Leaflet` for live maps, and are protected by
`HttpOnly`+`SameSite` session cookies with double-submit CSRF tokens.

On review (2026-08-12), a security and functionality audit of these pages
found and fixed one real XSS gap, an unpinned CDN script version, several
stale API payload shapes left over from earlier backend changes, and a
few pieces of fabricated demo data (fake wallet balances, fake ratings) —
see the "Web Platformu Kararı" section of the root README.md for the full
list. With those fixed, the empty `web/` scaffold was deleted and these
pages were adopted as the project's official web platform.

## Decision (Superseding)

The web platform is vanilla HTML/CSS/JS served directly by the Rust
backend from `backend/apps/api-gateway/static/`, not a separate
React/TypeScript single-page application. See QAS-000008 for the current
architecture of this layer.

## Consequences

- **Positive:** zero extra build toolchain, zero `node_modules`, zero
  deployment step beyond `cargo build` — the web UI ships inside the same
  binary as the API; every page was already real and working, so this
  decision cost nothing to "undo" (there was no working React app to
  discard).
- **Negative:** no component reuse across pages (each `.html` file
  duplicates its own `<style>`/fetch helpers); no type-checked API client
  the way a real TypeScript app would have; a full admin-panel React
  rewrite — if ever wanted — is a large future undertaking, materially
  bigger than the entire mobile phase, because it means re-implementing
  screens that already work.
- **Neutral:** the `sdk/typescript/` package (a client SDK, not a web
  app) is unaffected by this decision.

## Alternatives Considered (at time of superseding)

- **Build the React platform from scratch as originally planned:**
  rejected for this phase — a bigger undertaking than the mobile phase
  that had just been completed, replacing pages that already worked.
- **Delete the vanilla pages too and go dark on web:** rejected — they
  are real, working, and already linked from the root README as the
  project's live web interfaces.

## References

- QAS-000008 (web platform architecture, rewritten to match this
  decision), QES-000005 (originally the TypeScript/React standard; see
  its own file for how it now applies, or doesn't, to this codebase).
- Root [README.md](../../README.md) "Web Platformu Kararı" section,
  `BACKEND_BACKLOG.md` "Web platform boundary notes" section.

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs, proposing React/TypeScript. |
| 0.2.0 | 2026-08-12 | Marked Superseded; documented the actual decision (vanilla HTML/JS, empty React scaffold deleted). |
