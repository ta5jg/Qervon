#!/usr/bin/env bash
#
# ==============================================================================
# File:           scripts/release.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-06
# Version:        0.1.0
#
# Description:
#   Release gate: runs the full test and lint suite and, when explicitly
#   requested with --tag <version>, tags a clean working tree with an
#   annotated tag. Refuses to tag without the explicit flag.
#
# Usage:
#   scripts/release.sh                # verify gates only
#   scripts/release.sh --tag 0.1.0    # verify gates and tag v0.1.0
#
# Specification:
#   QMI-000000, QAS-000001 through QAS-000006, QES-000002, QES-000006.
#
# License:
#   Qervon License v1.0 — see LICENSE in the repository root.
# ==============================================================================

set -eu

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

TAG=""
if [ "$#" -ge 2 ] && [ "$1" = "--tag" ]; then
    TAG="$2"
    if ! echo "$TAG" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$'; then
        echo "error: invalid semver tag: $TAG" >&2
        exit 1
    fi
    if [ "$#" -ne 2 ]; then
        echo "usage: $0 [--tag <version>]" >&2
        exit 1
    fi
elif [ "$#" -ne 0 ]; then
    echo "usage: $0 [--tag <version>]" >&2
    exit 1
fi

echo "running release gates..."
"$ROOT/scripts/run-tests.sh"
"$ROOT/scripts/run-lints.sh"
"$ROOT/scripts/check-architecture.sh"
"$ROOT/scripts/check-documentation.sh"
"$ROOT/scripts/check-security.sh"

echo "gates passed"

if [ -n "$TAG" ]; then
    if [ -n "$(git -C "$ROOT" status --porcelain)" ]; then
        echo "error: working tree is not clean; commit or stash before tagging" >&2
        exit 1
    fi
    git -C "$ROOT" tag -a "v$TAG" -m "Qervon release v$TAG"
    echo "tagged v$TAG"
else
    echo "no tag requested; use: $0 --tag <version>"
fi
