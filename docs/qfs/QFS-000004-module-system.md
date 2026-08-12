<!-- =============================================================================
File:           docs/qfs/QFS-000004-module-system.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The real module system: compile-time Cargo crates under modules/*, not
  a runtime-pluggable mechanism.

Specification:
  ADR-000007, QAS-000001, QFS-000002.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QFS-000004 — Module System

**Status: Implemented — compile-time, not runtime-pluggable** (see
QFS-000002 for why there is no kernel/plugin mechanism on top of this).

## What a "module" is in this codebase

A Cargo crate under `backend/modules/` (`orders`, `dispatch`, `couriers`,
`customers`, `fleet`, `billing`, `notifications`, `tracking`, `identity`)
— a thin façade re-exporting the application-layer services relevant to
one bounded context, added as a workspace member in
`backend/Cargo.toml`, and depended on directly by `apps/api-gateway`.

## Adding a new module

1. `cargo new --lib modules/<name>`, add it to the workspace `members`
   list.
2. Depend on `crates/domain`/`crates/application` for the actual logic —
   the module crate itself should stay thin (see QES-000002).
3. `apps/api-gateway` adds the new crate as a dependency and wires its
   routes into the router by hand in `http.rs`.

There is no module manifest, no module registry, no runtime
enable/disable — a module is compiled in or it isn't; removing one means
deleting the crate and its route registrations, not flipping a
configuration flag.

## Why this is enough today

At Qervon's current size (nine modules, one deployment target), the
"lightweight compile-time crate" approach gives real dependency-boundary
enforcement (a compile error if `couriers` reaches into `orders`'
internals) without the complexity of a genuine plugin-loading mechanism
that nothing currently needs (see QFS-000002/QFS-000005).

## References

- ADR-000007 (modular monolith, the decision this implements), QAS-000001
  (the layering rules modules must respect), QFS-000002 (why there's no
  kernel above this).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real compile-time module structure. |
