#!/usr/bin/env bash
set -euo pipefail

: "${DATABASE_URL:?DATABASE_URL is required}"
backup_dir=${QERVON_BACKUP_DIR:-/var/lib/qervon/backups}
timestamp=$(date -u +%Y%m%dT%H%M%SZ)
destination="$backup_dir/qervon-$timestamp.dump"

install -d -m 0700 "$backup_dir"
pg_dump --format=custom --no-owner --file="$destination" "$DATABASE_URL"
pg_restore --list "$destination" >/dev/null
printf '%s\n' "$destination"
