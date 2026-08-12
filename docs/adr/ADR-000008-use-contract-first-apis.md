<!-- =============================================================================
File:           docs/adr/ADR-000008-use-contract-first-apis.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Architecture Decision Record: a dedicated api-contracts crate is the
  single source of truth for wire DTOs; OpenAPI docs are generated from
  the same annotated Rust types, not hand-maintained separately.

Specification:
  QMI-000000, QAS-000005.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# ADR-000008 — Contract-First APIs via a Shared `api-contracts` Crate

- **Status:** Accepted — implemented.
- **Date:** 2026-08-05.
- **Deciders:** Irfan Gedik.

## Context

Three native mobile apps (iOS, Android) and a web layer all consume the
same backend contract. Without one authoritative definition of each
request/response shape, DTOs drift out of sync across clients (as
happened in practice — see ADR-000004/BACKEND_BACKLOG.md's note on the
web pages sending a stale `fare_amount_minor` field after the backend
contract changed underneath them).

## Decision

Define every request/response DTO once, in `backend/crates/api-contracts`,
annotated with `serde::{Serialize, Deserialize}` and (on the response
side) `utoipa::ToSchema` so the same struct definitions drive both runtime
JSON (de)serialization and the generated OpenAPI document served at
`/api-docs/openapi.json` / `/swagger-ui`. `apps/api-gateway`'s HTTP
handlers only ever construct/consume these shared types — no handler
hand-builds ad-hoc `serde_json::Value` responses for a documented
endpoint.

This is "contract-first" in the sense that matters here: one Rust source
of truth generates the OpenAPI contract, rather than a hand-written OpenAPI
YAML file drifting from the code, or each client guessing the shape from
undocumented behavior. It is not schema-first in the stricter sense of an
OpenAPI/protobuf file that generates the Rust types — the direction is
Rust struct → OpenAPI doc, not the reverse.

## Consequences

- **Positive:** native clients (iOS `QervonCore`, Android `core:common`)
  were both written by directly reading `api-contracts`' struct
  definitions field-for-field, which is far more reliable than working
  from a separately-maintained spec; `/swagger-ui` is always in sync with
  the actual handlers because it's generated from the same types they use.
- **Negative:** no automatic client-code generation from the OpenAPI
  document exists yet — each native app's DTOs are hand-written to match
  `api-contracts`, so a future contract change still requires updating
  three places (backend, iOS, Android) by hand. A generator (e.g.
  `openapi-generator` targeting Swift/Kotlin) would close this gap but has
  not been adopted.
- **Neutral:** the TypeScript SDK under `sdk/typescript/` is a separate,
  hand-maintained client and is subject to the same manual-sync
  limitation.

## Alternatives Considered

- **A separately-maintained OpenAPI YAML file** driving Rust codegen:
  rejected — adds a build step and a second source of truth to keep in
  sync with the actual handler code.
- **No shared contracts crate, DTOs defined inline per handler:** this
  was the state that led to the drift problem this ADR fixes; rejected.

## References

- [QAS-000005](../qas/QAS-000005-api-integration-standard.md) (API/integration standard).
- [backend/crates/api-contracts/src/lib.rs](../../backend/crates/api-contracts/src/lib.rs).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the actual api-contracts crate and utoipa-based OpenAPI generation. |
