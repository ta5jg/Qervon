#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
temp_dir="$(mktemp -d)"
trap 'rm -rf "$temp_dir"' EXIT

valid_env="$temp_dir/valid.env"
cat > "$valid_env" <<'EOF'
QERVON_STORAGE=postgres
DATABASE_URL=postgres://qervon:password@localhost/qervon
QERVON_TOKEN_SIGNING_SECRET=0123456789abcdef0123456789abcdef
QERVON_API_ACCESS_TOKEN=separate-access-token
QERVON_WEBHOOK_ENCRYPTION_KEY=AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8=
QERVON_LISTEN=127.0.0.1:8080
EOF

"$root/scripts/production-preflight.sh" "$valid_env"

unsafe_env="$temp_dir/unsafe.env"
sed 's/QERVON_LISTEN=127.0.0.1:8080/QERVON_LISTEN=0.0.0.0:8080/' "$valid_env" > "$unsafe_env"
if "$root/scripts/production-preflight.sh" "$unsafe_env"; then
    echo 'Expected public bind to be rejected.' >&2
    exit 1
fi

short_secret_env="$temp_dir/short-secret.env"
sed 's/QERVON_TOKEN_SIGNING_SECRET=.*/QERVON_TOKEN_SIGNING_SECRET=short/' "$valid_env" > "$short_secret_env"
if "$root/scripts/production-preflight.sh" "$short_secret_env"; then
    echo 'Expected short signing secret to be rejected.' >&2
    exit 1
fi

invalid_webhook_key_env="$temp_dir/invalid-webhook-key.env"
sed 's/QERVON_WEBHOOK_ENCRYPTION_KEY=.*/QERVON_WEBHOOK_ENCRYPTION_KEY=invalid/' "$valid_env" > "$invalid_webhook_key_env"
if "$root/scripts/production-preflight.sh" "$invalid_webhook_key_env"; then
    echo 'Expected invalid webhook encryption key to be rejected.' >&2
    exit 1
fi

echo 'Production preflight tests passed.'
