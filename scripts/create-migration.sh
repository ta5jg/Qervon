#!/usr/bin/env bash
#
# ==============================================================================
# File:           scripts/create-migration.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-06
# Version:        0.1.0
#
# Description:
#   Scaffolds a new governed SQL migration under backend/migrations.
#
# Usage:
#   scripts/create-migration.sh <schema> <snake_case_name>
#   Example: scripts/create-migration.sh orders 00000000000002_add_eta
#
# Specification:
#   QMI-000000, QAS-000006, QES-000002, QES-000006.
#
# License:
#   Qervon License v1.0 — see LICENSE in the repository root.
# ==============================================================================

set -eu

if [ "$#" -ne 2 ]; then
    echo "usage: $0 <schema> <snake_case_name>" >&2
    exit 1
fi

SCHEMA="$1"
NAME="$2"

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MIGRATIONS_DIR="$ROOT/backend/migrations/$SCHEMA"

if ! echo "$NAME" | grep -qE '^[a-z0-9_]+$'; then
    echo "error: name must be lowercase snake_case (letters, digits, underscores)" >&2
    exit 1
fi

mkdir -p "$MIGRATIONS_DIR"

STAMP="$(date -u +%Y%m%d%H%M%S)"
FILE="$MIGRATIONS_DIR/${STAMP}_${NAME}.sql"

if [ -e "$FILE" ]; then
    echo "error: $FILE already exists" >&2
    exit 1
fi

cat > "$FILE" <<EOF
-- =============================================================================
-- File:           backend/migrations/$SCHEMA/${STAMP}_${NAME}.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   $(date -u +%Y-%m-%d)
-- Version:        0.1.0
--
-- Description:
--   Describe what this migration changes and why.
--
-- Specification:
--   QAS-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

EOF

echo "created $FILE"
