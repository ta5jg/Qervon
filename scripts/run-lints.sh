#!/usr/bin/env bash
#
# ==============================================================================
# File:           scripts/run-lints.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-05
# Version:        0.1.0
#
# Description:
#   Runs the Qervon backend formatters and linters.
#
# Specification:
#   QMI-000000, QAS-000001 through QAS-000006, QES-000002, QES-000006.
#
# License:
#   Qervon License v1.0 — see LICENSE in the repository root.
# ==============================================================================

set -eu

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

cd "$ROOT/backend"

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
