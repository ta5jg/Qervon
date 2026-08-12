<!-- =============================================================================
File:           docs/qfs/QFS-000007-workflow-engine.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  No generic workflow engine exists. Real, hardcoded state machines
  exist per domain (Order, Assignment) — explains the distinction.

Specification:
  QLS-000002, QLS-000003, QFS-000001.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QFS-000007 — Workflow Engine

**Status: Vision / Not Implemented** (as a generic engine). Real,
hardcoded state machines exist per domain — see below.

## What "workflow engine" would mean

A generic system where a workflow (a sequence of states and allowed
transitions, possibly with conditional branches, timeouts, and
compensating actions) is defined as *data* (a workflow definition file
or database rows) and executed by a shared interpreter — so a new
workflow can be added or changed without a code deploy.

## What actually exists

Two real, well-tested state machines, each hand-written in Rust as plain
`match`/enum-based transition methods on the domain type itself:

- **Order lifecycle** (`qervon_domain::order`, see QLS-000002):
  `Pending → CourierAssigned → InTransit → Delivered`, or
  `→ Cancelled`/`→ Returned`.
- **Assignment (offer/accept/reject)** (`qervon_domain::dispatch`, see
  QLS-000003): `Offered → Accepted/Rejected/Cancelled`, with a
  time-based expiry.

Both are workflows in the informal sense, but changing either means
editing Rust code and shipping a new binary — there is no data-driven
definition, no shared interpreter, and no way to add a third kind of
workflow without writing a third hand-built state machine.

## Why this hasn't been generalized

Two workflows is not (yet) enough repetition to justify extracting a
generic engine — doing so now would be speculative, and Rust's type
system already gives strong illegal-state-prevention for these two cases
for free (an invalid enum variant simply can't be constructed). A
generic engine would trade that compile-time safety for runtime
flexibility neither current workflow actually needs. Revisit if/when a
third or fourth genuinely distinct workflow (e.g. the vision-stage Field
Service domain, QLS-000012) is built and the duplication becomes real.

## References

- QLS-000002, QLS-000003 (the two real state machines), QFS-000001
  (series overview).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten distinguishing the real per-domain state machines from the absent generic engine. |
