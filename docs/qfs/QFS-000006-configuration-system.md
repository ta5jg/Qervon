<!-- =============================================================================
File:           docs/qfs/QFS-000006-configuration-system.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.3.0

Description:
  Configuration is entirely environment variables, read once at startup
  — no config file, no dynamic reconfiguration, no config service.

Specification:
  QFS-000003, QAS-000014.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QFS-000006 — Configuration System

**Status: Implemented — deliberately minimal.**

## Mechanism

Every configurable value is a `QERVON_*` environment variable, read once
in `AppState::from_env()` (QFS-000003) at process startup. There is no
config file format (no TOML/YAML config), no runtime config-reload, and
no external configuration service (no Consul/etcd/AWS Parameter Store
integration).

## Real variables in active use

| Variable | Purpose |
| --- | --- |
| `QERVON_STORAGE` | `memory` or `postgres` — selects the persistence backend (QAS-000006) |
| `QERVON_LISTEN` | bind address, default `0.0.0.0:8080` |
| `QERVON_TOKEN_SIGNING_SECRET` | HMAC key for access tokens (QAS-000004) |
| `QERVON_API_ACCESS_TOKEN` | static bearer token for machine-to-machine API access |
| `QERVON_CORS_ALLOWED_ORIGINS` | comma-separated allowed origins (QAS-000004) |
| `QERVON_WEBHOOK_ENCRYPTION_KEY` | encrypts stored webhook secrets at rest |
| `QERVON_WEB_PUSH_VAPID_PUBLIC_KEY` | web-push VAPID key (QLS-000010) |
| `QERVON_INITIAL_SETUP_TOKEN` | gates the `/setup` bootstrap page in production |
| `QERVON_WORKER_POLL_SECONDS` | the background worker's poll interval (QFS-000011) |
| `QERVON_UPLOADS_DIR` | local-filesystem root for uploaded files (delivery-proof photos), default `./data/uploads` — must be a persistent, backed-up path in production (QLS-000013) |
| `DATABASE_URL` | PostgreSQL connection string (when `QERVON_STORAGE=postgres`) |
| `RUST_LOG` | `tracing` env-filter |

See `/etc/qervon/qervon.env` in the deployment runbook (QAS-000014) for
which of these are mandatory in production.

## Why not a config file

At this system's current operational size, environment variables are
sufficient and match how the systemd unit files
(`infrastructure/systemd/qervon-api.service`) already inject
configuration — adding a config-file parser would be a second mechanism
solving the same problem, with no current need driving the added
flexibility (e.g. nested/structured config) a file format would offer.

## References

- [QFS-000003](QFS-000003-runtime-lifecycle.md) (when these are read), [QAS-000014](../qas/QAS-000014-deployment-architecture.md) (deployment — where
  these are actually set), [QAS-000004](../qas/QAS-000004-security-architecture.md) (security-relevant variables).

---

# Revision History

| Version | Date | Description |
| --------- | ------ | ------------- |
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real, complete list of active configuration variables. |
| 0.3.0 | 2026-08-13 | Added `QERVON_UPLOADS_DIR` (delivery-photo upload storage root). |
