#!/usr/bin/env bash
#
# ==============================================================================
# File:           scripts/check-security.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-06
# Version:        0.1.0
#
# Description:
#   Scans tracked files for common secret material and forbidden artifact
#   types. Scans only git-tracked files so build and dependency output is
#   never inspected. Test fixtures (Kotlin/Swift/Rust test directories) are
#   excluded from the password/token/secret heuristic, the same way
#   docs/*.md/*.sql already were, because they are expected to contain
#   obviously-fake credentials (e.g. "fakesignature", "supersecretpassword")
#   exercising encoding/parsing logic, not real secret material. The private
#   key and AWS credential patterns still scan everywhere, including tests,
#   since a real key accidentally committed there would be just as dangerous.
#
# Specification:
#   QMI-000000, QAS-000001 through QAS-000006, QES-000004.
#
# License:
#   Qervon License v1.0 — see LICENSE in the repository root.
# ==============================================================================

set -eu

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

fail=0

always_scanned_patterns=(
    '-----BEGIN (RSA |EC |OPENSSH )?PRIVATE KEY-----'
    'AKIA[0-9A-Z]{16}'
    '(AWS_ACCESS_KEY_ID|AWS_SECRET_ACCESS_KEY|AWS_SESSION_TOKEN)\s*[:=]\s*["'\'' ]?[^"'\'' ]+'
)

test_fixture_excluded_patterns=(
    '(^|[[:space:]])(password|passwd|secret|api[_-]?key|token)[[:space:]]*[:=][[:space:]]*["'\''][^"'\''[:space:]]{8,}'
)

exclude_pathspecs=(':!docs/**' ':!*.md' ':!*.sql')
test_dir_pathspecs=(
    ':!**/test/**' ':!**/tests/**' ':!**/Tests/**' ':!**/androidTest/**'
)

for pattern in "${always_scanned_patterns[@]}"; do
    matches="$(git grep -n -I -E -e "$pattern" -- . "${exclude_pathspecs[@]}" || true)"
    if [ -n "$matches" ]; then
        echo "POTENTIAL SECRET FOUND:"
        echo "$matches"
        fail=1
    fi
done

for pattern in "${test_fixture_excluded_patterns[@]}"; do
    matches="$(git grep -n -I -E -e "$pattern" -- . "${exclude_pathspecs[@]}" "${test_dir_pathspecs[@]}" || true)"
    if [ -n "$matches" ]; then
        echo "POTENTIAL SECRET FOUND:"
        echo "$matches"
        fail=1
    fi
done

forbidden_extensions=(
    '*.pem'
    '*.key'
    '*.p12'
    '*.keystore'
)

for ext in "${forbidden_extensions[@]}"; do
    matches="$(git ls-files "$ext" || true)"
    if [ -n "$matches" ]; then
        echo "FORBIDDEN ARTIFACT TRACKED: $ext"
        echo "$matches"
        fail=1
    fi
done

if [ "$fail" -ne 0 ]; then
    echo "security check FAILED" >&2
    exit 1
fi

echo "security check passed"
