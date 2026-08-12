<!-- =============================================================================
File:           docs/qas/QAS-000014-deployment-architecture.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The real deployment topology: a single VPS, systemd-managed binaries,
  a reverse proxy terminating TLS. Step-by-step procedure lives in the
  deployment runbook, not duplicated here.

Specification:
  QAS-000006, QES-000012.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000014 — Deployment Architecture

**Status: Implemented.** Full step-by-step procedure:
[docs/operations/deployment-runbook.md](../operations/deployment-runbook.md)
(this document covers the *shape*, not the *steps*).

## Topology

One VPS, no container orchestration, no Kubernetes:

```text
Internet → Caddy/Nginx (TLS termination, :443) → 127.0.0.1:8080 (qervon-api, systemd)
                                                 → PostgreSQL (local or managed, not publicly exposed)
qervon-worker (systemd) — background jobs (webhook delivery outbox, etc.), same binaries family
```

- `apps/api-gateway` compiles to a single dependency-free binary,
  deployed under `/opt/qervon/bin/`, run as a non-login `qervon` system
  user via a systemd unit (`infrastructure/systemd/qervon-api.service`).
- `apps/migration-runner` runs once per deploy, before restarting the API,
  applying any new SQL migrations (see QAS-000006).
- `apps/worker` runs continuously as its own systemd unit for
  fire-and-forget background jobs (e.g. webhook delivery retries).
- The reverse proxy (Caddy or Nginx) is the only public-facing process;
  PostgreSQL is never exposed beyond localhost/the VPS's private network.

## Configuration

All runtime configuration is environment variables in
`/etc/qervon/qervon.env`, readable only by the `qervon` user — no
secrets manager, no `.env` committed to git (see QES-000011). Mandatory
in production: `QERVON_STORAGE=postgres`, `DATABASE_URL`,
`QERVON_TOKEN_SIGNING_SECRET` (32+ chars), `QERVON_WEBHOOK_ENCRYPTION_KEY`,
`QERVON_API_ACCESS_TOKEN`.

## Release and rollback

`scripts/build-release.sh` builds all release binaries; the previous
API binary is kept as `.previous` on the VPS so a bad deploy can be
rolled back by stopping the service, restoring `.previous`, and
restarting — no blue/green, no canary, no automatic rollback trigger;
this is a manual, operator-verified process (see the runbook's
"Verification and rollback" section).

## Why deliberately no Docker/Kubernetes for the running API

Docker is used only for optional *local* development services (Postgres,
Redis) — never to run the actual API in this deployment model. The
choice was: budget, and predictability of a single static binary versus
container-orchestration overhead this project's team size and traffic
don't yet justify. Revisit if/when horizontal scaling across multiple
hosts becomes necessary — the binary itself has no code-level obstacle
to running in a container if that becomes the right call later.

## References

- [docs/operations/deployment-runbook.md](../operations/deployment-runbook.md)
  (the actual procedure), [QAS-000006](QAS-000006-database-persistence-standard.md) (migrations), [QES-000012](../qes/QES-000012-release-engineering.md) (release
  engineering practices).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten to describe the real single-VPS/systemd topology, pointing to the detailed runbook. |
