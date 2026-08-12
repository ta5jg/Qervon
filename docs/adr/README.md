<!-- =============================================================================
File:           docs/adr/README.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Index of Qervon's Architecture Decision Records.

Specification:
  QMI-000000, QMI-000003.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# Architecture Decision Records

Each ADR captures one binding technical decision: its context, the
decision itself, its consequences (including negative ones), and the
alternatives considered. ADRs are immutable once accepted — a changed
decision gets a **Superseded** status and a note pointing to what
replaced it (see ADR-000004, ADR-000006), rather than a silent rewrite.

| ADR | Title | Status |
| --- | --- | --- |
| [ADR-000001](ADR-000001-use-rust-for-backend.md) | Use Rust for the backend | Accepted |
| [ADR-000002](ADR-000002-use-kotlin-for-android.md) | Use Kotlin/Jetpack Compose for Android | Accepted |
| [ADR-000003](ADR-000003-use-swift-for-ios.md) | Use Swift/SwiftUI for iOS | Accepted |
| [ADR-000004](ADR-000004-use-react-typescript-for-web.md) | Web platform technology: React/TypeScript | **Superseded** — vanilla HTML/JS adopted instead |
| [ADR-000005](ADR-000005-use-postgresql-postgis.md) | Use PostgreSQL (PostGIS not adopted) | Accepted (partial) |
| [ADR-000006](ADR-000006-use-nats-jetstream.md) | Event bus: NATS JetStream | **Not Adopted** — pg_notify + broadcast channel used instead |
| [ADR-000007](ADR-000007-use-modular-monolith-first.md) | Modular monolith first | Accepted |
| [ADR-000008](ADR-000008-use-contract-first-apis.md) | Contract-first APIs via a shared crate | Accepted |
| [ADR-000009](ADR-000009-use-uuid-v7.md) | Use UUIDv7 for entity identifiers | Accepted |
| [ADR-000010](ADR-000010-use-hybrid-event-architecture.md) | Hybrid event architecture (narrow, not pervasive) | Accepted |

## Writing a new ADR

1. Copy the structure used by the files above: Status/Date/Deciders,
   Context, Decision, Consequences (Positive/Negative/Neutral),
   Alternatives Considered, References, Revision History.
2. Number it sequentially (`ADR-0000NN`); never reuse or renumber an
   existing ID.
3. If it changes an earlier decision, mark the earlier ADR's status as
   Superseded with a pointer to the new one — don't delete or silently
   edit the old decision's history.

## References

- [QMI-000003](../qmi/QMI-000003-architecture-governance.md) (architecture governance) for when an ADR is required.

---

## Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten as a real index with status tracking. |
