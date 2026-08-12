#!/usr/bin/env python3
"""Fix common markdown format issues across the Qervon repository."""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
PDF_DUMP_MARKER = "Bu dosya Qervon projesi dokümantasyonu"
HEADER_MARKER = "<!-- ============================================================================="
GOVERNANCE_TITLE = re.compile(r"^# (QMI|QAS|QES|QLS|QFS|ADR)-\d+ —")
ABS_PATH = re.compile(r"/Users/irfangedik/Qervon_Platform/qervon/([^)\s\]]+)")
HEADER_FIELDS = (
    "File:",
    "Project:",
    "Author:",
    "Developer:",
    "Created Date:",
    "Version:",
    "Description:",
    "Specification:",
    "License:",
)


def strip_pdf_dump(content: str) -> str:
    if PDF_DUMP_MARKER not in content:
        return content
    if HEADER_MARKER in content:
        return content[content.index(HEADER_MARKER) :]

    ursl = content.find("<!-- URSL:BEGIN")
    if ursl != -1:
        return content[ursl:]

    skill_front_matter = re.search(r"\n---\nname:", content)
    if skill_front_matter:
        return content[skill_front_matter.start() + 1 :]

    generic_start = re.search(r"\n---\n\n(# |<!-- URSL)", content)
    if generic_start:
        return content[generic_start.start() + 5 :]

    return content


def normalize_header_comment(content: str) -> str:
    for field in HEADER_FIELDS:
        content = re.sub(rf"^ {re.escape(field)}", field, content, flags=re.MULTILINE)
    return content


def fix_absolute_links(content: str, file_path: Path) -> str:
    def replacer(match: re.Match[str]) -> str:
        target = REPO_ROOT / match.group(1)
        return Path(os.path.relpath(target, file_path.parent)).as_posix()

    return ABS_PATH.sub(replacer, content)


def fix_runbook_step_headings(content: str) -> str:
    lines = content.splitlines()
    out: list[str] = []
    in_steps = False

    for line in lines:
        if line == "## Steps":
            in_steps = True
            out.append(line)
            continue

        if in_steps and line.startswith("## ") and line not in {
            "## Operational Baseline",
            "## References",
            "## Revision History",
        }:
            out.append("#" + line)
            continue

        if line.startswith("## ") and line in {
            "## Operational Baseline",
            "## References",
            "## Revision History",
        }:
            in_steps = False

        out.append(line)

    return "\n".join(out)


def fix_heading_hierarchy(content: str) -> str:
    lines = content.splitlines()
    out: list[str] = []
    seen_title = False
    is_governance = False

    for line in lines:
        if line.startswith("# ") and not line.startswith("##"):
            title = line[2:].strip()
            if GOVERNANCE_TITLE.match(line):
                is_governance = True
                seen_title = True
                out.append(line)
            elif not seen_title:
                seen_title = True
                out.append(line)
            elif is_governance and title == "Revision History":
                out.append(line)
            else:
                out.append(f"## {title}")
        else:
            out.append(line)

    return "\n".join(out)


def process_file(path: Path) -> bool:
    original = path.read_text(encoding="utf-8")
    updated = original
    updated = strip_pdf_dump(updated)
    updated = normalize_header_comment(updated)
    updated = fix_absolute_links(updated, path)
    updated = fix_heading_hierarchy(updated)
    updated = updated.replace("docs/sources/qervon-1.pdf", "docs/qervon-1.md")
    updated = updated.replace("docs/sources/qervon-2.pdf", "docs/qervon-2.md")
    updated = updated.replace("sources/qervon-1.pdf", "qervon-1.md")
    updated = updated.replace("sources/qervon-2.pdf", "qervon-2.md")
    updated = updated.replace("qervon-1.pdf", "qervon-1.md")
    updated = updated.replace("qervon-2.pdf", "qervon-2.md")
    updated = re.sub(
        r"^\| --- \| --- \| --- \|$",
        "|---------|------|-------------|",
        updated,
        flags=re.MULTILINE,
    )
    if "docs/operations" in path.as_posix():
        updated = fix_runbook_step_headings(updated)

    if path.name == "NOTICE.md":
        updated = updated.replace("[LICENSE.md](LICENSE.md)", "[LICENSE](LICENSE)")

    if updated != original:
        path.write_text(updated, encoding="utf-8")
        return True
    return False


def main() -> int:
    changed: list[str] = []
    for path in sorted(REPO_ROOT.rglob("*.md")):
        if process_file(path):
            changed.append(str(path.relative_to(REPO_ROOT)))

    print(f"Updated {len(changed)} markdown file(s).")
    for name in changed:
        print(f"  - {name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
