#!/usr/bin/env bash
#
# ==============================================================================
# File:           scripts/check-documentation.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-06
# Version:        0.1.0
#
# Description:
#   Verifies that the governed documentation baseline exists: root docs,
#   license, and the required specification families.
#
# Specification:
#   QMI-000000, QAS-000001 through QAS-000006.
#
# License:
#   Qervon License v1.0 — see LICENSE in the repository root.
# ==============================================================================

set -eu

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

required_files=(
    "$ROOT/README.md"
    "$ROOT/AGENTS.md"
    "$ROOT/LICENSE"
)

required_dirs=(
    "$ROOT/docs/qas"
    "$ROOT/docs/qes"
    "$ROOT/docs/qls"
    "$ROOT/docs/qfs"
    "$ROOT/docs/qmi"
    "$ROOT/docs/adr"
    "$ROOT/docs/rfc"
)

fail=0

for file in "${required_files[@]}"; do
    if [ ! -f "$file" ]; then
        echo "MISSING FILE: $file"
        fail=1
    fi
done

for dir in "${required_dirs[@]}"; do
    if [ ! -d "$dir" ]; then
        echo "MISSING DIRECTORY: $dir"
        fail=1
    elif [ -z "$(ls -A "$dir")" ]; then
        echo "EMPTY DIRECTORY: $dir"
        fail=1
    fi
done

if [ "$fail" -ne 0 ]; then
    echo "documentation check FAILED" >&2
    exit 1
fi

echo "documentation check passed"
