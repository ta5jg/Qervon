#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
env_file=${1:-/etc/qervon/qervon.env}
"$root/scripts/production-preflight.sh" "$env_file"
set -a
. "$env_file"
set +a
: "${DATABASE_URL:?DATABASE_URL missing from environment file.}"
"$root/scripts/backup-postgres.sh" >/dev/null
curl --fail --silent --show-error http://127.0.0.1:8080/ready >/dev/null
QERVON_LOAD_REQUESTS=${QERVON_LOAD_REQUESTS:-25} QERVON_LOAD_CONCURRENCY=${QERVON_LOAD_CONCURRENCY:-5} \
  "$root/scripts/load-smoke.sh" http://127.0.0.1:8080
echo 'Qervon production acceptance passed.'
