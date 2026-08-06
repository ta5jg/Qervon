#!/usr/bin/env bash
#
# ==============================================================================
# File:           scripts/start-development.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-05
# Version:        0.1.0
#
# Description:
#   Starts the Qervon API gateway for local development.
#   Uses in-memory storage by default; set QERVON_STORAGE=postgres and
#   DATABASE_URL to use PostgreSQL (migrations are applied automatically).
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

STORAGE="${QERVON_STORAGE:-memory}"

if [ "$STORAGE" = "postgres" ]; then
    if [ -z "${DATABASE_URL:-}" ]; then
        echo "DATABASE_URL is required when QERVON_STORAGE=postgres" >&2
        exit 1
    fi
    (cd "$BACKEND" && cargo run -q -p qervon-migration-runner)
fi

(cd "$BACKEND" && QERVON_STORAGE="$STORAGE" cargo run -p qervon-api-gateway)
