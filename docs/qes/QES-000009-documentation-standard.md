<!-- =============================================================================
File:           docs/qes/QES-000009-documentation-standard.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The file-header format used across every source and documentation
  file, and the rules for governance-document content specifically.

Specification:
  QMI-000001, QES-000001.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QES-000009 — Documentation Standard

## The file header (applies to every file in this repository)

```text
File:           <path from repo root>
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   <YYYY-MM-DD, real creation date, never changed after>
Version:        <bumped on real content changes, see QMI-000002>

Description:
  <one paragraph: what this file is for>

Specification:
  <QAS-/QES-/QLS-/QMI-/ADR- IDs, comma-separated, that govern or explain this file>

License:
  Qervon License v1.0 — see LICENSE in the repository root.
```

Comment syntax adapts per language (`//` blocks for Rust/Kotlin/JS,
`<!-- -->` for Markdown/HTML, `#` for YAML/shell) but the fields are
identical. This is what makes "which governance document explains this
code" a grep-able question (`grep -r "QAS-000007" backend/ mobile/`)
rather than tribal knowledge.

## Markdown governance-document structure

Every QMI/QAS/QES/QLS/QFS/ADR document (see QMI-000001 for the full
rule set) has, after the header:

1. A `# <ID> — <Title>` heading.
2. An explicit status line for the whole document (`**Status:
   Implemented.**` / `**Status: Vision / Not Implemented.**` /
   `**Status: Superseded.**` — see QMI-000000's honesty policy).
3. Real content organized under `##` sections.
4. A `## References` section linking related document IDs.
5. A `# Revision History` table, appended to on every substantive change,
   never rewritten to erase prior entries.

## Code comments (not headers)

Comments explain *why*, not *what* — the "no obvious/narrating comments"
rule (`// increment the counter` above `i += 1` is never acceptable). A
comment is warranted when the code's intent isn't obvious from reading
it: a non-obvious constraint, a workaround for a specific bug, a
reference to the governance document explaining a business rule.

## References

- QMI-000001 (the governance-specific rules this document details),
  QES-000001 (the header-format principle).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real, actually-used header format and document structure. |
