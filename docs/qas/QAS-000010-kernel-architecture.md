<!-- =============================================================================
File:           docs/qas/QAS-000010-kernel-architecture.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Honest status check on the "kernel" concept from the source PDFs: no
  generic, pluggable runtime kernel exists in this codebase.

Specification:
  QAS-000001, QFS-000002.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000010 — Kernel Architecture

**Status: Vision / Not Implemented.**

## What the source PDFs describe

A generic "kernel" that hosts pluggable domain modules (Delivery, Fleet,
Warehouse, Dispatch, Field Service, Retail, Medical Logistics) as
interchangeable units on top of a shared runtime — implying a plugin
loading mechanism, a common module lifecycle, and module-to-module
communication primitives independent of any specific domain.

## What actually exists instead

There is no such generic kernel. What exists is the modular-monolith
crate structure described in QAS-000001/ADR-000007: a fixed set of Rust
crates (`modules/orders`, `modules/dispatch`, `modules/couriers`, ...)
compiled directly into one binary, wired together by hand in
`apps/api-gateway/src/state.rs`. Adding a new "module" today means
writing a new Cargo crate, adding it to the workspace, and wiring its
routes into the router by hand — there is no runtime plugin-loading, no
module manifest format, no dynamic enable/disable of a module without a
recompile and redeploy.

This is a deliberate simplification, not an oversight: at Qervon's
current scale (one modular monolith, a handful of bounded contexts,
one deployment target), building a genuine plugin kernel would be
speculative infrastructure with no current consumer. See QFS-000002 for
the fuller discussion of why the QFS "foundation kernel" series describes
vision rather than shipped code.

## What would need to be built for this to become real

- A defined module trait/interface (route registration, migration
  registration, background-task registration) that `apps/api-gateway`
  loads a *list* of, rather than hand-wiring each one.
- A decision on whether "pluggable" means compile-time (a Cargo feature
  flag per module, still one binary) or genuinely dynamic (loading a
  `.so`/`.dylib` at runtime, WASM modules, or separate processes per
  module) — each has very different complexity and safety tradeoffs, and
  none has been chosen.

## References

- [QAS-000001](QAS-000001-architecture-philosophy.md) (the real layering), [ADR-000007](../adr/ADR-000007-use-modular-monolith-first.md) (modular monolith), [QFS-000002](../qfs/QFS-000002-kernel-architecture.md)
  (the foundation-series kernel vision this document sits alongside).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as an explicit Vision/Not Implemented status with a pointer to what exists instead. |
