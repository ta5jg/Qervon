#!/usr/bin/env bash
set -euo pipefail

env_file=${1:-/etc/qervon/qervon.env}
test -r "$env_file" || { echo "Missing environment file: $env_file" >&2; exit 1; }
set -a
. "$env_file"
set +a

test "${QERVON_STORAGE:-}" = postgres || { echo 'QERVON_STORAGE must be postgres in production' >&2; exit 1; }
test -n "${DATABASE_URL:-}" || { echo 'DATABASE_URL is required' >&2; exit 1; }
test -n "${QERVON_TOKEN_SIGNING_SECRET:-}" || { echo 'QERVON_TOKEN_SIGNING_SECRET is required' >&2; exit 1; }
secret_length=${#QERVON_TOKEN_SIGNING_SECRET}
test "$secret_length" -ge 32 || { echo 'QERVON_TOKEN_SIGNING_SECRET must be at least 32 characters' >&2; exit 1; }
test -n "${QERVON_API_ACCESS_TOKEN:-}" || { echo 'QERVON_API_ACCESS_TOKEN is required' >&2; exit 1; }
test -n "${QERVON_WEBHOOK_ENCRYPTION_KEY:-}" || { echo 'QERVON_WEBHOOK_ENCRYPTION_KEY is required' >&2; exit 1; }
webhook_key_length="$(printf %s "$QERVON_WEBHOOK_ENCRYPTION_KEY" | openssl base64 -d -A 2>/dev/null | wc -c | tr -d ' ')"
test "$webhook_key_length" = 32 || { echo 'QERVON_WEBHOOK_ENCRYPTION_KEY must decode to 32 bytes' >&2; exit 1; }
test -n "${QERVON_LISTEN:-}" || { echo 'QERVON_LISTEN is required' >&2; exit 1; }
case "$QERVON_LISTEN" in
    127.0.0.1:*|\[::1\]:*) ;;
    *)
        echo 'QERVON_LISTEN must use loopback; terminate TLS at the reverse proxy' >&2
        exit 1
        ;;
esac
echo 'Qervon production preflight passed.'
