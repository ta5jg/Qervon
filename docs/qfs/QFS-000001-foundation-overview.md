<!-- =============================================================================
File:           docs/qfs/QFS-000001-foundation-overview.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Orientation for the QFS series: which of its 15 documents describe
  real, shipped infrastructure vs. vision for a generic extensible
  platform that does not exist yet.

Specification:
  QMI-000000, QAS-000001, QAS-000010.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QFS-000001 — Foundation Overview

## What "Foundation Specification" means here

The source architecture PDFs describe Qervon's long-term ambition as a
generic "Logistics Operating System" — a platform with a pluggable
kernel, a workflow engine, a policy/permission/rule engine, and a
scheduler, on top of which domain modules (Delivery, Fleet, Warehouse,
...) would run as interchangeable units. This QFS series documents that
vision, document by document — and, per this project's honesty policy
(QMI-000000), says plainly which parts are real today and which aren't.

## Status at a glance

| Doc | Topic | Status |
| --- | --- | --- |
| QFS-000002 | Kernel architecture | **Vision** — see QAS-000010 |
| QFS-000003 | Runtime lifecycle | Implemented (the real `api-gateway` startup/shutdown sequence) |
| QFS-000004 | Module system | Implemented (the real `modules/*` crate pattern) |
| QFS-000005 | Plugin system | **Vision** |
| QFS-000006 | Configuration system | Implemented (env-var based) |
| QFS-000007 | Workflow engine | **Vision** (real state machines exist per-domain, not a generic engine) |
| QFS-000008 | Policy engine | **Vision** (real RBAC exists, hardcoded per-endpoint, not a generic policy language) |
| QFS-000009 | Permission engine | Implemented (as hardcoded RBAC) — no generic engine |
| QFS-000010 | Rule engine | **Vision** (pricing/coupon rules are hardcoded Rust, not data-driven) |
| QFS-000011 | Scheduler | Implemented (a simple polling-based background worker) |
| QFS-000012 | Observability runtime | See QAS-000012 (duplicate topic, not repeated here) |
| QFS-000013 | AI gateway | **Vision** (AI logic runs in-process, see QAS-000009 — no separate gateway service) |
| QFS-000014 | Integration runtime | Implemented (partial — real webhooks) |
| QFS-000015 | Meta-platform roadmap | Vision/roadmap document by definition |

## Why this honesty matters more here than anywhere else

This series is the most tempting place in the whole documentation set to
describe an ambitious future architecture as if it already existed — the
words "kernel", "plugin", "engine" sound impressive and the source PDFs
use them liberally. Doing so would be the single most misleading thing
this documentation set could contain, since a new engineer reading
"Policy Engine" would reasonably expect a real, callable system. Every
QFS document below states its real status in its first line for exactly
this reason.

## References

- QMI-000000 (the honesty policy this series follows most carefully),
  QAS-000001/QAS-000010 (the real architecture vs. this vision).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as an honest status map for the entire QFS series. |
