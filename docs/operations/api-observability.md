# API Observability

The API exposes three unauthenticated operational endpoints intended for a
loopback-only or reverse-proxy-restricted operations network:

| Endpoint | Contract |
| --- | --- |
| `GET /health` | Process liveness. Returns `200` when the HTTP process is running. |
| `GET /ready` | Readiness for traffic. Returns `503` until an API authentication mechanism is configured. |
| `GET /metrics` | Prometheus text metrics for request outcomes, response-time accumulation, uptime, location count, and authentication configuration. |

Every HTTP response has a server-generated `X-Request-Id`, plus
`X-Content-Type-Options: nosniff` and `Referrer-Policy: no-referrer`. Request
completion logs include only the request identifier, method, status, and
duration; they deliberately omit authorization headers, request bodies, query
strings, and paths.

The API rejects request bodies over 1 MiB before JSON handlers consume them.

`/health` is a liveness probe, not a database integrity check. `/ready` confirms
that the API is configured to authenticate requests; PostgreSQL connectivity is
validated during process startup. Monitor database-level availability separately
until an active pooled-connection probe is introduced.
