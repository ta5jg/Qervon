#!/usr/bin/env bash
set -euo pipefail

root=${QERVON_ROOT:-/opt/qervon}
env_file=${QERVON_ENV_FILE:-/etc/qervon/qervon.env}
"$root/scripts/production-preflight.sh" "$env_file"
set -a
. "$env_file"
set +a
: "${DATABASE_URL:?DATABASE_URL missing from environment file.}"
"$root/scripts/backup-postgres.sh" >/dev/null
MIGRATIONS_DIR="${MIGRATIONS_DIR:-$root/migrations}" "$root/bin/qervon-migration-runner"
systemctl daemon-reload
systemctl restart qervon-api
systemctl restart qervon-worker
systemctl enable --now qervon-backup.timer
sleep 2
curl --fail --silent --show-error http://127.0.0.1:8080/health >/dev/null
systemctl is-active --quiet qervon-api
systemctl is-active --quiet qervon-worker
systemctl is-enabled --quiet qervon-backup.timer
"$root/scripts/production-acceptance.sh" "$env_file"
echo 'Qervon deployment verified.'
