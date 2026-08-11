#!/usr/bin/env bash
#
# ==============================================================================
# File:           scripts/stop-development.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-06
# Version:        0.1.0
#
# Description:
#   Stops the locally running Qervon development services (API gateway and
#   migration runner). Does not touch external services such as PostgreSQL.
#
# Specification:
#   QMI-000000, QAS-000001 through QAS-000006, QES-000002, QES-000006.
#
# License:
#   Qervon License v1.0 — see LICENSE in the repository root.
# ==============================================================================

set -eu

PIDS="$(pgrep -f 'qervon-(api-gateway|migration-runner|worker)' || true)"

if [ -z "$PIDS" ]; then
    echo "no Qervon development processes are running"
    exit 0
fi

echo "stopping Qervon development processes: $PIDS"
kill $PIDS 2>/dev/null || true
sleep 1

REMAINING="$(pgrep -f 'qervon-(api-gateway|migration-runner|worker)' || true)"
if [ -n "$REMAINING" ]; then
    echo "forcing stop: $REMAINING" >&2
    kill -9 $REMAINING 2>/dev/null || true
fi

echo "stopped"
