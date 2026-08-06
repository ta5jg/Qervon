#!/usr/bin/env bash
#
# ==============================================================================
# File:           scripts/run-benchmarks.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-06
# Version:        0.1.0
#
# Description:
#   Runs the backend benchmark suite when benchmark targets are configured.
#   Reports honestly when no benchmarks exist instead of failing.
#
# Specification:
#   QMI-000000, QAS-000001 through QAS-000006, QES-000002, QES-000006.
#
# License:
#   Qervon License v1.0 — see LICENSE in the repository root.
# ==============================================================================

set -eu

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND="$ROOT/backend"

if ! grep -rqE '\[\[bench\]\]|#\[bench\]' "$BACKEND" --include='Cargo.toml' --include='*.rs' 2>/dev/null; then
    echo "no benchmark targets configured in $BACKEND; skipping"
    exit 0
fi

(cd "$BACKEND" && cargo bench --workspace)
