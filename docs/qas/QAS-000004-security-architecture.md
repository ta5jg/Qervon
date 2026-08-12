<!-- =============================================================================
File:           docs/qas/QAS-000004-security-architecture.md
Project:        Qervon
Author:         USDTG GROUP TECHNOLOGY LLC
Developer:      Irfan Gedik
Created Date:   2026-08-05
Version:        0.2.0

Description:
  Authentication, authorization, transport, and abuse-mitigation controls
  actually implemented in the backend and web layer.

Specification:
  QAS-000011, QES-000014.

License:
  Qervon License v1.0 — see LICENSE in the repository root.
============================================================================= -->

# QAS-000004 — Security Architecture

**Status: Implemented.**

## Authentication

- **Password:** `argon2` hashing (`POST /v1/auth/login`,
  `/v1/auth/register`), never stored or logged in plaintext.
- **Access tokens:** a custom `qv1.<base64url payload>.<hmac signature>`
  format (see `backend/apps/api-gateway/src/auth.rs`), HMAC-SHA256 signed
  with `QERVON_TOKEN_SIGNING_SECRET`. Contains `subject` (user id),
  `tenant_id`, `role`, `expires_at` — 15 minutes lifetime. This is a
  bespoke scheme, not a JWT library, kept deliberately simple and
  auditable.
- **Refresh tokens:** stored server-side only as a hash
  (`hash_refresh_token`), never in plaintext in the database; rotated on
  every refresh; invalidated on logout.
- **Browser (cookie) sessions:** `qervon_access_token` (`HttpOnly`,
  `SameSite=Lax`), `qervon_refresh_token` (`HttpOnly`, `SameSite=Strict`,
  scoped to `/v1/browser/auth`), `qervon_csrf_token` (readable by JS, for
  the double-submit CSRF pattern below). All three get the `Secure`
  attribute when the request arrived over HTTPS (detected via
  `X-Forwarded-Proto` behind a reverse proxy).
- **Native mobile:** Bearer token in the `Authorization` header, stored
  client-side in the platform Keychain (iOS) / EncryptedSharedPreferences
  (Android) — never in plain `UserDefaults`/`SharedPreferences`.
- **OTP (phone) login:** `POST /v1/auth/otp/request` +
  `/v1/auth/otp/verify`; the code is real but is **not sent via SMS** in
  this environment — see BACKEND_BACKLOG.md.

## CSRF

Cookie-authenticated (browser) requests use the double-submit pattern:
the CSRF cookie's value must match an `X-Csrf-Token` header on every
non-GET/HEAD/OPTIONS request (`csrf_is_valid` in `http.rs`). Bearer-token
(mobile) requests are exempt — CSRF is a cookie-specific attack, and
mobile clients never send the auth cookie.

## Authorization

Role-based, checked per-handler via middleware (`require_operational_access`,
`require_courier_access`, `require_signed_user`, ...) plus explicit
ownership checks in handlers (`require_customer_order`,
`require_courier_order`) that verify the resource actually belongs to the
tenant/user making the request — a valid token for tenant A's order does
not grant access to tenant B's order, even for the same role. See
QAS-000011.

## Transport-level controls

- **CORS** (`cors_layer()` in `http.rs`): origins are read from
  `QERVON_CORS_ALLOWED_ORIGINS` (comma-separated); with none configured,
  cross-origin browser requests are rejected while the same-origin shipped
  HTML pages are unaffected. Credentialed requests allowed, methods
  limited to `GET/POST/PUT/PATCH/DELETE`, headers limited to
  `Content-Type`/`Authorization`.
- **Rate limiting** (`tower_governor`, IP-keyed via a custom
  `ClientIpKeyExtractor`): a tight tier on credential-bearing endpoints
  (register/login/OTP — 10 requests per 3-second window) and a looser
  global tier on everything else (120 requests per 200ms window).
- **`usesCleartextTraffic`/local dev:** the local/dev default is plain
  HTTP; production behind `silhor.com` is expected to terminate TLS at
  a reverse proxy — see the deployment runbook.

## Web-layer application security

XSS: every governance-relevant string rendered into `innerHTML` on the
shipped web pages must go through the shared `escapeHtml()` helper — one
real gap here (an unescaped order-address label in
`mobile-customer.html`) was found and fixed on 2026-08-12; see QAS-000008.

## What is not implemented

- No WAF, no DDoS mitigation beyond the rate limiter above.
- No secrets manager integration — secrets are environment variables set
  on the VPS (see the deployment runbook and QES-000014).
- No automated dependency vulnerability scanning wired into CI yet (see
  QES-000010/QES-000011).

## References

- [QAS-000011](QAS-000011-multi-tenant-architecture.md) (multi-tenant isolation), [QES-000014](../qes/QES-000014-secure-coding-standard.md) (secure coding
  standard), [BACKEND_BACKLOG.md](../../BACKEND_BACKLOG.md) (OTP/SMS gap).

---

# Revision History

| Version | Date | Description |
|---------|------|-------------|
| 0.1.0 | 2026-08-05 | Placeholder generated from source PDFs. |
| 0.2.0 | 2026-08-12 | Rewritten with the real auth token format, CORS/rate-limit config, and CSRF scheme. |
