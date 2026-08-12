<!-- =============================================================================
File:           docs/qfs/QFS-000011-scheduler.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The real background-job mechanism: a simple polling worker with a
  claim-lock pattern, not a cron scheduler or a distributed job queue.

Specification:
  QFS-000003, QLS-000010.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QFS-000011 — Scheduler

**Status: Implemented — a simple polling worker, not cron or a
distributed job queue.** Source: `backend/apps/worker/src/main.rs`.

## Mechanism

`apps/worker` runs as its own long-lived systemd process (QAS-000014),
looping forever:

```text
loop {
    process_pending_webhook_deliveries();   // and similar outbox tables
    sleep(QERVON_WORKER_POLL_SECONDS);      // default 5 seconds
}
```

Each pending item (e.g. a webhook delivery, see QFS-000014) is claimed
via a `claimed_at` timestamp column with a 5-minute reclaim window — if
a claimed item hasn't completed within 5 minutes (crashed worker, hung
delivery), a subsequent poll cycle re-claims and retries it. This is the
"outbox pattern": work is written to a database table inside the same
transaction as the triggering event, and a separate process reliably
drains that table — not a fire-and-forget in-process background task
that could be lost if the API process restarts.

## What this is not

- **Not cron** — there is no concept of "run this at 3am daily"; the
  worker continuously polls at a fixed interval for *any* pending work,
  regardless of when it became pending.
- **Not a distributed job queue** (no Sidekiq/Celery/BullMQ equivalent)
  — there is exactly one worker process type today, polling its own
  fixed set of outbox tables; there is no generic "enqueue an arbitrary
  job" API for other code to use.
- **Not horizontally scaled** — running two `qervon-worker` processes
  against the same database would both poll the same tables; the
  claim-lock pattern prevents double-processing the *same* row, but
  there's no work-distribution/sharding beyond that.

## References

- QFS-000003 (the worker's startup lifecycle, same shape as api-gateway),
  QFS-000014 (webhooks — the concrete outbox this worker drains),
  QLS-000010 (notifications — another candidate consumer of this
  pattern).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real polling/claim-lock mechanism and explicit non-goals. |
