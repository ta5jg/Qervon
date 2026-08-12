<!-- =============================================================================
File:           docs/api/idempotency-policy.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.1.0

Description:
   Defines the API guidance for idempotency policy.

Specification:
   QMI-000000 and QAS-000005 API integration standard

License:
   Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# Idempotency Policy

## Purpose

This document defines idempotency policy for the Qervon API surface.

## Guidance

- Require idempotency for operations that may be retried.
- Use stable keys and clear error responses.
- Keep repeated requests safe and predictable.

## References

- [qervon-2.md](../qervon-2.md)
- [docs/qas/QAS-000005-api-integration-standard.md](../qas/QAS-000005-api-integration-standard.md)

---

## Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Aligned idempotency policy to the source PDFs. |
