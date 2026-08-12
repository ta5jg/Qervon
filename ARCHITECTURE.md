<!-- =============================================================================
File:           ARCHITECTURE.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.1.0

Description:
   Summarizes the canonical Qervon architecture direction and the product-to-platform relationship.

Specification:
   QMI-000000 and the Qervon source PDFs

License:
   Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# Architecture

Qervon is designed as a modular Logistics Operating System, not a single-purpose courier app.

## Purpose

This document summarizes the architectural direction established in the source PDFs and points readers to the governing specifications.

## Core Direction

- Separate platform foundation from business domains.
- Use clean, layered, and contract-first design.
- Prefer explicit boundaries over shared implementation.
- Keep the system observable, secure, and scalable by default.

## Primary References

- [qervon-1.md](docs/qervon-1.md)
- [qervon-2.md](docs/qervon-2.md)
- [docs/qmi/QMI-000000-master-architecture-index.md](docs/qmi/QMI-000000-master-architecture-index.md)

---

## Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Aligned architecture summary to the source PDFs. |
