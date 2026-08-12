<!-- =============================================================================
File:           docs/qfs/QFS-000005-plugin-system.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  No plugin system exists. Explains what would be needed if one were
  ever built, and why it hasn't been a priority.

Specification:
  QFS-000001, QFS-000004.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QFS-000005 — Plugin System

**Status: Vision / Not Implemented.**

## What exists instead

New functionality is added as a new Cargo crate under `modules/*`
(QFS-000004), compiled into the binary, with routes hand-wired by a
developer — there is no way for a third party (or even an internal team)
to add functionality without a recompile of `api-gateway` itself, and no
mechanism to load code at runtime (no dynamic library loading, no WASM
module runtime, no separate-process plugin protocol).

## What "real" would require

Any of these, each with very different tradeoffs, none chosen:

- **WASM plugins** — sandboxed, safe, but limited in what host
  capabilities (database access, etc.) can be exposed without a
  significant host-function API design effort.
- **Dynamic library loading** (`.so`/`.dylib` via `libloading`) — fast,
  but unsafe (no sandboxing) and version-fragile (ABI compatibility
  between host and plugin).
- **Separate-process plugins** (gRPC/Unix-socket) — safest and most
  flexible, but reintroduces the deployment/operational complexity
  ADR-000007 deliberately avoided by choosing a modular monolith.

## Why this has not been built

No current requirement needs third-party or dynamically-loaded
extensibility — every "module" so far (QFS-000004) has been written by
the same team that maintains the core. Building speculative plugin
infrastructure before a real consumer exists for it would be exactly the
kind of premature architecture this project's engineering principles
(QES-000001) warn against.

## References

- [QFS-000001](QFS-000001-foundation-overview.md) (series overview), [QFS-000004](QFS-000004-module-system.md) (the real module system this
  would extend), [ADR-000007](../adr/ADR-000007-use-modular-monolith-first.md) (modular monolith).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as an explicit Vision/Not Implemented status with the real tradeoffs any future implementation would face. |
