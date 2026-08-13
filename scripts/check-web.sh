#!/usr/bin/env bash
#
# ==============================================================================
# File:           scripts/check-web.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-13
# Version:        0.1.0
#
# Description:
#   Guards the official web platform (backend/apps/api-gateway/static/) against
#   two concrete regressions that were manually fixed once already (see
#   README.md "Web Platformu Kararı"): unpinned third-party CDN scripts (a
#   supply-chain risk) and plaintext HTTP references to external resources.
#   Does not attempt full HTML validation or XSS static analysis — those need
#   a real parser and are intentionally out of scope for a dependency-free
#   shell check.
#
# Specification:
#   QMI-000000, QAS-000008, QES-000014.
#
# License:
#   Qervon License v1.0 -- see LICENSE in the repository root.
# ==============================================================================

set -eu

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATIC_DIR="$ROOT/backend/apps/api-gateway/static"

if [ ! -d "$STATIC_DIR" ]; then
    echo "MISSING DIRECTORY: $STATIC_DIR"
    exit 1
fi

fail=0

html_files="$(find "$STATIC_DIR" -maxdepth 1 -type f -name '*.html' | sort)"

if [ -z "$html_files" ]; then
    echo "NO HTML FILES FOUND under $STATIC_DIR"
    exit 1
fi

for file in $html_files; do
    unpinned="$(grep -nE 'src="https://[^"]*@latest[^"]*"' "$file" || true)"
    if [ -n "$unpinned" ]; then
        echo "UNPINNED CDN SCRIPT in $file:"
        echo "$unpinned"
        fail=1
    fi

    plaintext_http="$(grep -nE '(src|href)="http://[^"]+"' "$file" || true)"
    if [ -n "$plaintext_http" ]; then
        echo "PLAINTEXT HTTP RESOURCE in $file:"
        echo "$plaintext_http"
        fail=1
    fi
done

if [ "$fail" -ne 0 ]; then
    echo "web check FAILED" >&2
    exit 1
fi

echo "web check passed ($(echo "$html_files" | wc -l | tr -d ' ') files scanned)"
