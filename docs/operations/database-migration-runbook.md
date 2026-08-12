<!-- =============================================================================
File:           docs/operations/database-migration-runbook.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.1.0

Description:
   Defines the Qervon runbook for database migration runbook.

Specification:
   QMI-000000 and QAS-000014 deployment architecture

License:
   Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# Database Migration Runbook

## Purpose

This runbook defines how to execute and verify database migration runbook safely in the Qervon platform.

## Steps

- Plan schema changes before execution.
- Confirm backup and rollback readiness.
- Verify post-migration health before closing the change.

## Operational Baseline

- Use clear owners and approval gates.
- Verify the result before closing the task.
- Record any exception or rollback path.

## References

- [qervon-2.md](../qervon-2.md)
- [docs/operations/README.md](README.md)

---

## Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Aligned database migration runbook to the source PDFs. |
