#!/usr/bin/env python3
"""Convert plain-text doc-ID mentions inside '## References' / '# References'
sections into real relative markdown links, e.g. 'QES-000006' ->
'[QES-000006](../qes/QES-000006-testing-standard.md)'.

Scope is deliberately limited to References sections so prose elsewhere
that merely *names* a document ID (without intending a jump link) is left
untouched.
"""

from __future__ import annotations

import os
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
ID_RE = re.compile(r"\b((?:ADR|QAS|QES|QFS|QLS|QMI|RFC)-\d{6})\b")
ALREADY_LINKED_ID_RE = re.compile(r"\[[^\]]*\]\([^)]*\)")
BACKLOG_RE = re.compile(r"(?<!\]\()(?<!/)\bBACKEND_BACKLOG\.md\b(?!\))")
REFERENCES_HEADING_RE = re.compile(r"^#{1,2}\s+References\s*$", re.MULTILINE)
NEXT_HEADING_RE = re.compile(r"^#{1,2}\s+\S", re.MULTILINE)


def build_id_index() -> dict[str, Path]:
    index: dict[str, Path] = {}
    for md in REPO_ROOT.rglob("*.md"):
        m = ID_RE.search(md.name)
        if m:
            index[m.group(1)] = md
    return index


def _masked_ranges(segment: str) -> list[tuple[int, int]]:
    return [m.span() for m in ALREADY_LINKED_ID_RE.finditer(segment)]


def _in_masked(pos: int, masked_spans: list[tuple[int, int]]) -> bool:
    return any(start <= pos < end for start, end in masked_spans)


def linkify_segment(segment: str, current_file: Path, id_index: dict[str, Path]) -> str:
    # Recompute masked (already-linked) spans before *each* substitution
    # pass, since replacing text changes string length/offsets and stale
    # spans from a prior pass would misalign and corrupt later matches.
    def replace_id(m: re.Match[str], masked_spans: list[tuple[int, int]]) -> str:
        if _in_masked(m.start(), masked_spans):
            return m.group(0)
        idv = m.group(1)
        target = id_index.get(idv)
        if target is None or target == current_file:
            return m.group(0)
        rel = Path(os.path.relpath(target, current_file.parent)).as_posix()
        return f"[{idv}]({rel})"

    masked = _masked_ranges(segment)
    segment = ID_RE.sub(lambda m: replace_id(m, masked), segment)

    def replace_backlog(m: re.Match[str], masked_spans: list[tuple[int, int]]) -> str:
        if _in_masked(m.start(), masked_spans):
            return m.group(0)
        target = REPO_ROOT / "BACKEND_BACKLOG.md"
        if target == current_file:
            return m.group(0)
        rel = Path(os.path.relpath(target, current_file.parent)).as_posix()
        return f"[BACKEND_BACKLOG.md]({rel})"

    masked = _masked_ranges(segment)
    segment = BACKLOG_RE.sub(lambda m: replace_backlog(m, masked), segment)
    return segment


def process_file(path: Path, id_index: dict[str, Path]) -> bool:
    text = path.read_text(encoding="utf-8")
    heading_match = REFERENCES_HEADING_RE.search(text)
    if not heading_match:
        return False

    start = heading_match.end()
    next_heading_match = NEXT_HEADING_RE.search(text, start)
    end = next_heading_match.start() if next_heading_match else len(text)

    segment = text[start:end]
    new_segment = linkify_segment(segment, path, id_index)
    if new_segment == segment:
        return False

    new_text = text[:start] + new_segment + text[end:]
    path.write_text(new_text, encoding="utf-8")
    return True


def main() -> int:
    id_index = build_id_index()
    changed = []
    for md in sorted(REPO_ROOT.rglob("*.md")):
        if process_file(md, id_index):
            changed.append(str(md.relative_to(REPO_ROOT)))

    print(f"Updated {len(changed)} file(s).")
    for name in changed:
        print(f"  - {name}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
