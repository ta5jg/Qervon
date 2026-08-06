#!/usr/bin/env bash
#
# ==============================================================================
# File:           scripts/bootstrap-development.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-06
# Version:        0.1.0
#
# Description:
#   Prepares a local machine for Qervon backend development: verifies the
#   pinned Rust toolchain, installs required components, and warms the build.
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

if ! command -v cargo >/dev/null 2>&1; then
    echo "error: cargo not found; install Rust via https://rustup.rs" >&2
    exit 1
fi

(cd "$BACKEND" && rustup component add rustfmt clippy)
(cd "$BACKEND" && cargo check --workspace --all-targets)

echo "bootstrap complete: $(cargo --version)"
