<!-- =============================================================================
File:           docs/qas/QAS-000012-observability-architecture.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  What actually exists for logs, metrics, and health checks — structured
  tracing plus a hand-written Prometheus-text /metrics endpoint, no
  tracing-span-export or dashboarding stack.

Specification:
  QAS-000014.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000012 — Observability Architecture

**Status: Implemented (basic) — no distributed tracing, no dashboard
stack.**

## Logs

`tracing` + `tracing-subscriber` (env-filter) emit structured logs to
stdout; every completed HTTP request logs `request_id`, `method`,
`status`, `duration_ms` via the `observe_request` middleware in
`http.rs`. `request_id` is a UUIDv7 generated per request, useful for
correlating a single request's log lines but not propagated to any
downstream trace-collection system (there is no OpenTelemetry exporter).

## Metrics

`GET /metrics` returns hand-written Prometheus text-exposition format
(`metrics_handler` in `http.rs`) — no `prometheus`/`metrics` crate,
just `format!()`. Real gauges today: `qervon_live_courier_locations`
(count of couriers this process currently holds a location for),
`qervon_auth_configured` (0/1), and an HTTP-requests-by-status-class
counter. There is no scraping/alerting stack (Prometheus server,
Grafana, Alertmanager) configured in this repository — the endpoint
exists for an operator to wire one up.

## Health checks

- `GET /health` — liveness, always 200 while the process is up.
- `GET /ready` — readiness, 200 only once authentication is configured
  (`QERVON_API_ACCESS_TOKEN` or `QERVON_TOKEN_SIGNING_SECRET` set), 503
  otherwise; also reports the active storage backend.

See [docs/operations/api-observability.md](../operations/api-observability.md)
for the operator-facing runbook using these endpoints.

## What is not implemented

- No distributed tracing (no span export to Jaeger/Tempo/etc.).
- No log aggregation shipped by default (stdout only; an operator is
  expected to pipe this into `journald`/a log shipper on the VPS).
- No APM (application performance monitoring) integration.
- No synthetic monitoring/uptime checks configured in this repo.

## References

- [QAS-000014](QAS-000014-deployment-architecture.md) (deployment — how logs/metrics reach an operator in
  practice), [docs/operations/api-observability.md](../operations/api-observability.md).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real logging/metrics/health-check implementation and explicit gaps. |
