#!/usr/bin/env python3
"""
Fail CI when backlog-only implementation markers drift from docs status.
"""

from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[1]

BACKLOG_ONLY_RE = re.compile(r"STATUS:\s*v2 backlog", re.IGNORECASE)
DOMAIN_WIRED_RE = re.compile(r"STATUS:\s*(wired|live|production)", re.IGNORECASE)


def scan_backlog_markers() -> list[Path]:
    candidates = [
        ROOT / "backend" / "crates" / "domain",
        ROOT / "backend" / "crates" / "application",
    ]
    marked: list[Path] = []
    for directory in candidates:
        for file_path in directory.rglob("*.rs"):
            content = file_path.read_text(encoding="utf-8")
            if BACKLOG_ONLY_RE.search(content):
                marked.append(file_path)
    return marked


def scan_status_docs() -> list[Path]:
    docs_root = ROOT / "docs"
    marked: list[Path] = []
    for file_path in docs_root.rglob("*.md"):
        content = file_path.read_text(encoding="utf-8")
        if DOMAIN_WIRED_RE.search(content):
            marked.append(file_path)
    return marked


def main() -> int:
    backlog_marked = scan_backlog_markers()
    status_docs = scan_status_docs()
    if backlog_marked and status_docs:
        print("status drift check failed:")
        print("  - backlog-only markers still exist in implementation files")
        for file_path in backlog_marked:
            print(f"    * {file_path.relative_to(ROOT)}")
        print("  - at least one governance/status doc reports wired/live status")
        for file_path in status_docs[:10]:
            print(f"    * {file_path.relative_to(ROOT)}")
        print("  resolve by removing backlog markers or updating status docs.")
        return 1

    print("status drift check passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
