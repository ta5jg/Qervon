#!/usr/bin/env bash
#
# ==============================================================================
# File:           scripts/create-adr.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-06
# Version:        0.1.0
#
# Description:
#   Scaffolds a new Architecture Decision Record (ADR) under docs/adr.
#
# Usage:
#   scripts/create-adr.sh <short_kebab_case_title>
#   Example: scripts/create-adr.sh use-redis-for-pub-sub
#
# Specification:
#   QMI-000000, QES-000002, QES-000006.
#
# License:
#   Qervon License v1.0 — see LICENSE in the repository root.
# ==============================================================================

set -eu

if [ "$#" -ne 1 ]; then
    echo "usage: $0 <short_kebab_case_title>" >&2
    exit 1
fi

TITLE="$1"

if ! echo "$TITLE" | grep -qE '^[a-z0-9]+(-[a-z0-9]+)*$'; then
    echo "error: title must be kebab-case (lowercase letters, digits, hyphens)" >&2
    exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ADR_DIR="$ROOT/docs/adr"
mkdir -p "$ADR_DIR"

NEXT="$(find "$ADR_DIR" -name 'ADR-*.md' -maxdepth 1 -print0 | xargs -0 -n1 basename 2>/dev/null \
    | grep -oE 'ADR-[0-9]+' | grep -oE '[0-9]+' | sort -n | tail -1 || true)"
NEXT="${NEXT:-0}"
NEXT=$((10#$NEXT + 1))
NUMBER="$(printf '%05d' "$NEXT")"

FILE="$ADR_DIR/ADR-$NUMBER-$TITLE.md"

if [ -e "$FILE" ]; then
    echo "error: $FILE already exists" >&2
    exit 1
fi

cat > "$FILE" <<EOF
<!-- =============================================================================
 File:           docs/adr/ADR-$NUMBER-$TITLE.md
 Project:        Qervon
 Author:         USDTG GROUP TECHNOLOGY LLC
 Developer:      Irfan Gedik
 Created Date:   $(date -u +%Y-%m-%d)
 Version:        0.1.0

 Description:
   Architecture Decision Record for: $TITLE.

 Specification:
   QMI-000000.

 License:
   Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# ADR-$NUMBER — $TITLE

# Status

Proposed

# Context

Describe the problem, forces, and constraints that motivate this decision.

# Decision

Describe the chosen approach and the alternatives that were considered.

# Consequences

- List the positive consequences.
- List the trade-offs and follow-up work.

# References

- Link to the governing QAS/QFS/QES/QMI documents that apply.

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | $(date -u +%Y-%m-%d) | Initial draft. |
EOF

echo "created $FILE"
