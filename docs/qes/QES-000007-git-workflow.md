<!-- =============================================================================
File:           docs/qes/QES-000007-git-workflow.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Branching, commit message, and PR conventions actually used.

Specification:
  QES-000008.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000007 — Git Workflow

## Branching

`main` is the trunk. Feature work happens on a short-lived branch (e.g.
`android-kurye-musteri`) and merges back via PR; there is no long-lived
`develop`/`release` branch split at this project's current size.

## Commit messages

A commit message states *why* a change was made, not just *what* changed
(the diff already shows *what*). Style used throughout this repository's
history: a short imperative summary line, followed by a body explaining
the motivation and, for larger changes, a bullet list of the concrete
sub-changes. Example shape (not a literal past commit):

```text
Add server-side fare quoting to prevent client-controlled pricing

Customer order creation previously accepted a client-supplied
fare_amount_minor/fare_currency pair, meaning a modified client could
set its own price. Adds DeliveryPricing + PricingService and makes
POST /v1/customer/orders always compute the authoritative fare
server-side from pickup/dropoff distance.
```

## Pull requests

`.github/pull_request_template.md` exists and should be filled in
completely, not left as boilerplate — the CODEOWNERS file
(`.github/CODEOWNERS`) is real and assigns review responsibility. Issue
templates exist for architecture changes, features, and bugs
(`.github/ISSUE_TEMPLATE/`).

## What is not enforced

There is no branch-protection-rule documentation in this repository
(GitHub branch protection settings live in repo settings, not in git) —
if/when configured, this document should be updated to say what's
required (passing CI, N approvals, etc.) rather than left silent on it.

## References

- QES-000008 (code review standard), QES-000010 (CI/CD — what actually
  gates a merge today).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real branching/commit conventions used in this repository's history. |
