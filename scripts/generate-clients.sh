#!/usr/bin/env bash
#
# ==============================================================================
# File:           scripts/generate-clients.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-06
# Version:        0.1.0
#
# Description:
#   Generates API client code when a contract-first OpenAPI specification
#   exists. The backend currently has no published OpenAPI artifact, so this
#   script reports that state instead of pretending to generate clients.
#
# Specification:
#   QAS-000004, QAS-000005, QES-000005, QES-000008.
#
# License:
#   Qervon License v1.0 — see LICENSE in the repository root.
# ==============================================================================

set -eu

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

SPEC=""
for candidate in "$ROOT/docs/api/openapi.yaml" "$ROOT/docs/api/openapi.json" "$ROOT/docs/api/openapi.yml"; do
    if [ -f "$candidate" ]; then
        SPEC="$candidate"
        break
    fi
done

if [ -z "$SPEC" ]; then
    echo "no OpenAPI specification found under docs/api; client generation is not wired yet"
    echo "publish an OpenAPI artifact (e.g. docs/api/openapi.yaml) to enable generation"
    exit 0
fi

echo "found spec: $SPEC"
echo "client generation for this spec is not wired yet"
exit 0
