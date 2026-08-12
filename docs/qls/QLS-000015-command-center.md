<!-- =============================================================================
File:           docs/qls/QLS-000015-command-center.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  The admin dashboard (index.html): what an operator can actually see
  and do from it.

Specification:
  QAS-000008, QAS-000009, QAS-000011.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QLS-000015 — Command Center

**Status: Implemented.** The "command center" is the real admin
dashboard at `/`/`/index.html` (see QAS-000008 for its technical
architecture).

## What an operator can see and do

- **Live map + heatmap:** every courier's current location within their
  tenant (Leaflet), a manually-toggled sample heatmap layer, and an AI
  Fraud Guard tab listing any currently-flagged speed anomalies (see
  QAS-000009).
- **Metrics bar:** active orders, available/busy couriers, pending
  orders, delivered revenue (by currency), in-transit count — all
  computed live from `GET /v1/operations/overview` and related
  endpoints, not hardcoded.
- **Order management:** a live order table with an inline "assign
  courier" / "start transit" action per row where applicable.
- **Courier fleet table**, **finance/wallet report**, and **company/team
  management** (provision a courier, a tenant admin, or an entirely new
  tenant — role-gated to the appropriate level, see QAS-000011).

## What it does not do

- No bulk operations (bulk-assign, bulk-cancel).
- No saved custom views/dashboards per operator.
- No export-to-CSV/report-download feature.
- No audit log view (who provisioned which courier, when) — actions are
  recorded in the database (creation timestamps exist) but there is no
  dedicated audit-trail UI surfacing them.

## References

- QAS-000008 (the page's technical architecture and security controls),
  QAS-000009 (the AI Fraud Guard tab), QAS-000011 (the tenant-scoping
  every view in this dashboard respects).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real dashboard capabilities and explicit scope gaps. |
