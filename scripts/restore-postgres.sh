#!/usr/bin/env bash
set -euo pipefail

: "${DATABASE_URL:?DATABASE_URL is required}"
backup_file=${1:?usage: restore-postgres.sh /path/to/qervon.dump}
test -r "$backup_file" || { echo "Backup is not readable: $backup_file" >&2; exit 1; }
test "${QERVON_RESTORE_CONFIRM:-}" = restore || {
  echo 'Set QERVON_RESTORE_CONFIRM=restore after stopping API and worker services.' >&2
  exit 1
}

pg_restore --list "$backup_file" >/dev/null
pg_restore --clean --if-exists --no-owner --exit-on-error --dbname="$DATABASE_URL" "$backup_file"
echo 'PostgreSQL restore completed.'
