# VPS Release Runbook

Qervon production runs directly on the host. Docker is optional for local
PostgreSQL and Redis only; it is not part of the runtime deployment.

## Server layout

Create the dedicated service account and persistent locations once:

```sh
sudo useradd --system --home /opt/qervon --shell /usr/sbin/nologin qervon
sudo install -d -o qervon -g qervon /opt/qervon/bin /var/lib/qervon
sudo install -d -o root -g qervon -m 0750 /etc/qervon
```

Install the repository at `/opt/qervon`, then install
`infrastructure/systemd/qervon-api.service` as
`/etc/systemd/system/qervon-api.service`.

## Environment

Create `/etc/qervon/qervon.env` with mode `0640`, owned by `root:qervon`.
It must contain, at minimum:

```dotenv
QERVON_STORAGE=postgres
DATABASE_URL=postgres://...
QERVON_TOKEN_SIGNING_SECRET=<at-least-32-character-secret>
QERVON_API_ACCESS_TOKEN=<separate-service-secret>
QERVON_LISTEN=127.0.0.1:8080
```

Keep the API on loopback. A TLS reverse proxy is the only public entry point;
do not expose port 8080 to the internet.

## Release

From the checked-out release revision, build the two release binaries:

```sh
sudo -u qervon QERVON_BINARY_DIR=/opt/qervon/bin /opt/qervon/scripts/build-release.sh
sudo /opt/qervon/scripts/deploy-vps.sh
```

`deploy-vps.sh` refuses unsafe configuration, applies migrations before the
service restart, verifies the local health endpoint, and checks that systemd
kept the service active. Take and verify a database backup before every
migration-bearing release; use `docs/operations/backup-restore-runbook.md`.

On the first installation, enable the service after copying the unit:

```sh
sudo systemctl daemon-reload
sudo systemctl enable --now qervon-api
```

## Verification and recovery

After a release, inspect service status and recent logs:

```sh
sudo systemctl status qervon-api
sudo journalctl -u qervon-api -n 100 --no-pager
curl --fail http://127.0.0.1:8080/health
```

If the application health check fails, stop the rollout, restore the previous
release binaries, restart the service, and restore the database only according
to the backup/restore runbook. Do not roll back database migrations by
guessing or editing production tables manually.
