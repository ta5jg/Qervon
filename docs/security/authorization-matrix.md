<!-- =============================================================================
File:           docs/security/authorization-matrix.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.1.0

Description:
   Defines the Qervon security guidance for authorization matrix.

Specification:
   QMI-000000 and QAS-000004 security architecture

License:
   Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# Authorization Matrix

## Purpose

This document defines authorization matrix for the Qervon platform.

## Guidance

- Map roles to allowed actions explicitly.
- Keep permission checks server-side.
- Document exceptions and escalation paths.

## Common Security Baseline

- Use zero trust and least privilege.
- Keep authorization server-side and auditable.
- Protect sensitive data with clear handling rules.

## References

- [qervon-2.md](../qervon-2.md)
- [docs/qas/QAS-000004-security-architecture.md](../qas/QAS-000004-security-architecture.md)

---

## Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Aligned authorization matrix to the source PDFs. |
