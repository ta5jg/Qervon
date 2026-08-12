<!-- =============================================================================
File:           docs/qas/QAS-000005-api-integration-standard.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  HTTP API conventions: routing, error shape, pagination (or its
  absence), and how clients discover the contract.

Specification:
  ADR-000008, QAS-000004.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000005 — API Integration Standard

**Status: Implemented.**

## Routing

- Every endpoint is under `/v1/...`; there is no `/v2` yet (see
  QMI-000002 on API versioning policy).
- Resource-oriented paths (`/v1/customer/orders/{id}/cancel`,
  `/v1/courier/orders/{id}/accept`) rather than RPC-style verbs baked
  into the path where a REST verb+path fits naturally; a few
  action-shaped exceptions exist where "accept/reject/cancel" reads more
  clearly than a generic `PATCH` with a status field (see the full route
  table in `backend/apps/api-gateway/src/http.rs`).
- Query parameters for read-only filtering/computation (e.g.
  `GET /v1/customer/fare-quote?pickup_latitude=...`), never for anything
  that mutates state.

## Error shape

Every error response is `{"status": <int>, "title": <string>, "detail":
<string>}` (`ApiError`/`ApiErrorBody` in `api-contracts`), with the HTTP
status code matching the `status` field. Clients (all three: iOS,
Android, and the web pages) decode this shape uniformly rather than
guessing at error text.

## Discovery

`GET /api-docs/openapi.json` and `/swagger-ui` are generated from the
same `utoipa`-annotated types the handlers actually use (see
ADR-000008) — they are not a hand-maintained spec that can drift from
the real behavior.

## What is deliberately absent

- **No pagination** on any list endpoint today (`GET /v1/orders`,
  `/v1/couriers`, etc. return the full tenant-scoped set). Acceptable at
  current data volumes; will need `?cursor=`/`?limit=` parameters before
  any tenant's order history grows into the tens of thousands.
- **No GraphQL, no WebSocket-for-everything** — the source PDFs mention
  GraphQL as an option; only REST + one narrow WebSocket path
  (`/ws/tracking`, see QAS-000003) exist.
- **No API-key-based third-party integration surface** — `/v1/customer/webhooks`
  exists for outbound event delivery to a customer's own endpoint, but
  there is no inbound public API-key system for third parties to call
  Qervon directly yet.

## Client conventions

- Native mobile clients (iOS `QervonNetworking`, Android `core:network`)
  both implement the same pattern: an `HttpClient`/`OkHttpClient` wrapper
  that attaches the Bearer token, and a one-shot refresh-and-retry on a
  single 401 (never an infinite retry loop) — see QAS-000007.
- The shipped web pages attach the CSRF header and cookie credentials on
  every request via a small shared `api()`/`adminApi()` fetch wrapper
  per page (see QAS-000008).

## References

- ADR-000008 (contract-first via `api-contracts`), QAS-000004 (auth on
  top of this transport), QAS-000007 (mobile client conventions).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real routing/error/discovery conventions and explicit scope gaps. |
