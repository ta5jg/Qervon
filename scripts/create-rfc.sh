#!/usr/bin/env bash
#
# ==============================================================================
# File:           scripts/create-rfc.sh
# Project:        Qervon
# Author:         USDTG GROUP TECHNOLOGY LLC
# Developer:      Irfan Gedik
# Created Date:   2026-08-06
# Version:        0.1.0
#
# Description:
#   Scaffolds a new Request for Comments (RFC) under docs/rfc.
#
# Usage:
#   scripts/create-rfc.sh <short_kebab_case_title>
#   Example: scripts/create-rfc.sh notification-pipeline
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
RFC_DIR="$ROOT/docs/rfc"
mkdir -p "$RFC_DIR"

NEXT="$(find "$RFC_DIR" -name 'RFC-*.md' -maxdepth 1 -print0 | xargs -0 -n1 basename 2>/dev/null \
    | grep -oE 'RFC-[0-9]+' | grep -oE '[0-9]+' | sort -n | tail -1 || true)"
NEXT="${NEXT:-0}"
NEXT=$((10#$NEXT + 1))
NUMBER="$(printf '%05d' "$NEXT")"

FILE="$RFC_DIR/RFC-$NUMBER-$TITLE.md"

if [ -e "$FILE" ]; then
    echo "error: $FILE already exists" >&2
    exit 1
fi

cat > "$FILE" <<EOF
<!-- =============================================================================
 File:           docs/rfc/RFC-$NUMBER-$TITLE.md
 Project:        Qervon
 Author:         USDTG GROUP TECHNOLOGY LLC
 Developer:      Irfan Gedik
 Created Date:   $(date -u +%Y-%m-%d)
 Version:        0.1.0

 Description:
   Request for Comments: $TITLE.

 Specification:
   QMI-000000.

 License:
   Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# RFC-$NUMBER — $TITLE

# Status

Draft

# Problem Statement

Describe the problem this RFC addresses and why it matters.

# Proposed Solution

Describe the proposed approach in enough detail to comment on.

# Open Questions

- List unresolved questions that need feedback.

# Alternatives Considered

- List rejected alternatives and the reasons.

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | $(date -u +%Y-%m-%d) | Initial draft. |
EOF

echo "created $FILE"
