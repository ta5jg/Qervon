<!-- =============================================================================
File:           docs/operations/deployment-runbook.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.1.0

Description:
   Defines the Qervon runbook for deployment runbook.

Specification:
   QMI-000000 and QAS-000014 deployment architecture

License:
   Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# Deployment Runbook

## Purpose

This runbook defines how to execute and verify deployment runbook safely in the Qervon platform.

## Steps

### Release preparation

1. Run `make check` from the repository root.
2. Build the release binaries with `scripts/build-release.sh`; this includes the API, migration runner and webhook outbox worker.
3. Back up PostgreSQL before applying a migration.
4. Copy the binaries to `/opt/qervon/bin/` on the VPS and keep the previous API binary as `.previous` for rollback.

### VPS configuration

1. Create a non-login `qervon` user and `/etc/qervon/qervon.env` readable only by that user.
2. Set `QERVON_STORAGE=postgres`, `DATABASE_URL`, a 32+ character random `QERVON_TOKEN_SIGNING_SECRET`, a base64-encoded 32-byte `QERVON_WEBHOOK_ENCRYPTION_KEY`, `QERVON_API_ACCESS_TOKEN`, `QERVON_LISTEN=127.0.0.1:8080`, and `RUST_LOG` in that environment file. These secrets are mandatory in every production runtime.
3. Set `QERVON_UPLOADS_DIR` to an absolute, persistent path outside `/opt/qervon/bin/` (e.g. `/var/lib/qervon/uploads`), owned by the `qervon` user — this is where uploaded delivery-proof photos are stored (see QLS-000013); it must survive binary redeploys. Unlike PostgreSQL (QAS-000015), this directory has no automated backup procedure yet — treat that as an open gap, not an assumption.
4. Install `infrastructure/systemd/qervon-api.service`, `infrastructure/systemd/qervon-worker.service`, `infrastructure/systemd/qervon-backup.service`, and `infrastructure/systemd/qervon-backup.timer` under `/etc/systemd/system/`.
5. Apply migrations with the migration runner before restarting the API.
6. Run `systemctl daemon-reload && systemctl enable --now qervon-api qervon-worker qervon-backup.timer`. The timer creates a PostgreSQL custom-format backup once daily at 02:15 with a bounded random delay; `Persistent=true` runs a missed backup after reboot.
7. Put Caddy or Nginx in front of `127.0.0.1:8080` to terminate TLS; do not expose PostgreSQL or Redis publicly.

### First tenant owner bootstrap

Run this once, directly on the VPS after migrations and before enabling normal user access. Do not put these values in the API environment file or shell history; use a protected one-time environment file and remove it immediately after success.

```bash
sudo -u qervon env \
  DATABASE_URL='postgres://...' \
  QERVON_BOOTSTRAP_ALLOW=confirm \
  QERVON_BOOTSTRAP_TENANT_NAME='Example Logistics' \
  QERVON_BOOTSTRAP_TENANT_SLUG='example-logistics' \
  QERVON_BOOTSTRAP_EMAIL='owner@example.com' \
  QERVON_BOOTSTRAP_PASSWORD='use-a-long-unique-password' \
  /opt/qervon/bin/qervon-bootstrap-admin
```

The command refuses to run without explicit confirmation and refuses to overwrite an existing email or tenant slug. It creates a global super-admin identity plus an `owner` membership for the requested tenant. Public registration can only create customer identities and cannot select a tenant or an elevated role.

### Browser setup alternative

For a fresh PostgreSQL installation, the same first owner can be created through the HTTPS-only `/setup` page. Set a unique, 16+ character `QERVON_INITIAL_SETUP_TOKEN` in the API service environment before starting the API. The page asks for this token together with the tenant and owner details, and it permanently closes after the first tenant exists. Remove the token from the service environment and restart the API after successful setup. For local `QERVON_STORAGE=memory` development, `/setup` is available without a token because the data is non-persistent and the process is bound to loopback.

### Verification and rollback

1. Check `systemctl status qervon-api`, `systemctl status qervon-worker`, `systemctl list-timers qervon-backup.timer`, and `journalctl -u qervon-api -n 100`.
2. Verify `curl http://127.0.0.1:8080/health` locally on the VPS before switching proxy traffic.
3. On failure, stop the service, restore the `.previous` binary, restart it, and investigate before reattempting the release.

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
| 0.1.0 | 2026-08-05 | Aligned deployment runbook to the source PDFs. |
| 0.2.0 | 2026-08-10 | Added direct binary and systemd VPS deployment procedure. |
