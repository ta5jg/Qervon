<!-- =============================================================================
File:           docs/qas/QAS-000015-disaster-recovery.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  What disaster recovery actually means at this system's current scale:
  a real PostgreSQL backup/restore procedure exists; there is no
  multi-region failover or automated recovery.

Specification:
  QAS-000006, QAS-000014.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000015 — Disaster Recovery

**Status: Implemented (data layer only) — no infrastructure-level
failover.**

## What exists

A real, operator-run backup and restore procedure for the one piece of
state that actually needs it — the PostgreSQL database (every other
piece of "state" is either derived, in-memory/ephemeral, or the binaries
themselves, which are just redeployed from source):

- `scripts/backup-postgres.sh` produces a PostgreSQL custom-format dump
  under `/var/lib/qervon/backups`, verified with `pg_restore --list`
  before being reported as successful.
- `infrastructure/systemd/qervon-backup.timer` runs that backup daily at
  02:15 with a bounded random delay. `Persistent=true` makes a missed run
  execute after the VPS returns from downtime.
- `scripts/restore-postgres.sh` requires an explicit
  `QERVON_RESTORE_CONFIRM=restore` guard and both services stopped first
  — a restore overwrites the target database and is never run casually.

Full procedure: [docs/operations/backup-restore-runbook.md](../operations/backup-restore-runbook.md).

## What this does not cover

- **No off-host backup transport automation** — encrypted backups are
  copied to "the approved off-host backup location" via manual VPS
  operations process; there is no S3/object-storage upload wired into
  the script.
- **No multi-region or multi-AZ failover** — one VPS, one PostgreSQL
  instance. A VPS-level outage (not just a process crash) means downtime
  until the operator provisions a replacement and restores the latest
  backup — there is no standby replica to fail over to.
- **No documented Recovery Time Objective (RTO)** — restore time depends
  on the database size and the replacement VPS provision time. The on-host
  backup schedule gives an expected RPO of at most one day, but a VPS loss
  can still lose every backup until off-host replication is configured.

## Recommended next steps (not yet implemented)

1. Automated off-host upload of the encrypted dump.
2. A documented RTO and a periodic
   restore-drill to verify backups are actually restorable (an untested
   backup is not a real disaster-recovery plan).

## References

- [docs/operations/backup-restore-runbook.md](../operations/backup-restore-runbook.md)
  (the real, current procedure), [QAS-000006](QAS-000006-database-persistence-standard.md) (what's being backed up),
  [QAS-000014](QAS-000014-deployment-architecture.md) (the deployment topology this recovers).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real backup/restore procedure and an honest list of what's not automated yet. |
| 0.3.0 | 2026-08-21 | Added the daily persistent systemd backup timer and its recovery boundary. |
