#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

test "${QERVON_RUN_POSTGRES_INTEGRATION_TESTS:-}" = confirm || {
    echo 'Set QERVON_RUN_POSTGRES_INTEGRATION_TESTS=confirm to run against the disposable test database.' >&2
    exit 1
}
test -n "${QERVON_TEST_DATABASE_URL:-}" || {
    echo 'QERVON_TEST_DATABASE_URL is required.' >&2
    exit 1
}

export DATABASE_URL="$QERVON_TEST_DATABASE_URL"
cd "$root/backend"
cargo run -p qervon-migration-runner
QERVON_TEST_DATABASE_URL="$QERVON_TEST_DATABASE_URL" \
    cargo test -p qervon-infrastructure --test postgres_repositories -- --ignored --test-threads=1
