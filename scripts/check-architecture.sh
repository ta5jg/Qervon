#!/usr/bin/env bash
#
# ==============================================================================
# File:           scripts/check-architecture.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-06
# Version:        0.1.0
#
# Description:
#   Enforces the backend module layering: a crate may only depend on crates at
#   or below its own layer. Domain is the bottom layer; application depends on
#   domain; infrastructure depends on domain; modules wrap application and
#   domain; apps may depend on anything.
#
# Specification:
#   QAS-000001 through QAS-000006, QFS-000002, QES-000002, QES-000006.
#
# License:
#   Qervon License v1.0 — see LICENSE in the repository root.
# ==============================================================================

set -eu

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BACKEND="$ROOT/backend"

fail=0

check_layer() {
    local manifest="$1"
    local allowed="$2"
    local crate_dir
    crate_dir="$(dirname "$manifest")"

    local deps
    deps="$(awk '/^\[dependencies\]/{f=1;next}/^\[/{f=0}f' "$manifest" \
        | grep -oE 'qervon-[a-z0-9-]+' | sort -u || true)"

    for dep in $deps; do
        if [ "$dep" = "qervon" ]; then
            continue
        fi
        if ! echo " $allowed " | grep -q " $dep "; then
            echo "ARCHITECTURE VIOLATION: $crate_dir depends on $dep which is not in its allowed set: $allowed"
            fail=1
        fi
    done
}

check_layer "$BACKEND/crates/domain/Cargo.toml" ""
check_layer "$BACKEND/crates/application/Cargo.toml" "qervon-domain"
check_layer "$BACKEND/crates/infrastructure/Cargo.toml" "qervon-domain"
check_layer "$BACKEND/crates/api-contracts/Cargo.toml" "qervon-domain"
check_layer "$BACKEND/crates/test-support/Cargo.toml" "qervon-domain"
check_layer "$BACKEND/modules/couriers/Cargo.toml" "qervon-application qervon-domain"
check_layer "$BACKEND/modules/dispatch/Cargo.toml" "qervon-application qervon-domain"
check_layer "$BACKEND/modules/orders/Cargo.toml" "qervon-application qervon-domain"

if [ "$fail" -ne 0 ]; then
    echo "architecture check FAILED" >&2
    exit 1
fi

echo "architecture check passed"
