// =============================================================================
// File:           backend/apps/api-gateway/tests/outbound_providers.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-13
// Version:        0.1.0
//
// Description:
//   Proves the pluggable outbound provider wiring for OTP SMS, payment
//   charge, and native push dispatch (BACKEND_BACKLOG.md's "Faz-2.1 scope
//   boundaries") is a real HTTP client integration, not a placeholder: each
//   test points the relevant `*_provider_url` at a tiny local axum server
//   and asserts the exact request (method, body, bearer token) that arrives.
//   What remains environment-dependent is only the real third-party account
//   and endpoint URL, which this repository intentionally does not have
//   credentials for — the call site itself is exercised end-to-end here.
//
// Specification:
//   QAS-000005, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use axum::{extract::State as AxumState, http::HeaderMap, routing::post, Json, Router};
use chrono::{Duration as ChronoDuration, Utc};
use http_body_util::BodyExt;
use qervon_api_gateway::auth::issue_access_token;
use qervon_api_gateway::http::router;
use qervon_api_gateway::state::AppState;
use qervon_domain::{
    PushPlatform, TenantCompany, TenantId, TenantMemberRole, TenantMembership, UserId, UserRole,
};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

/// A single request captured by the fake provider server.
#[derive(Clone, Debug)]
struct CapturedRequest {
    body: Value,
    authorization: Option<String>,
}

type Captured = Arc<Mutex<Vec<CapturedRequest>>>;

/// Starts a tiny local HTTP server standing in for a third-party provider
/// (SMS gateway, payment gateway, or push dispatcher). Returns its base URL
/// and a handle to every request it has received, so tests can assert on
/// the exact payload and bearer token the application sent.
async fn spawn_fake_provider() -> (String, Captured) {
    let captured: Captured = Arc::new(Mutex::new(Vec::new()));
    let captured_for_handler = Arc::clone(&captured);

    async fn capture(
        AxumState(captured): AxumState<Captured>,
        headers: HeaderMap,
        Json(body): Json<Value>,
    ) -> Json<Value> {
        let authorization = headers
            .get("authorization")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        captured
            .lock()
            .expect("captured lock")
            .push(CapturedRequest {
                body,
                authorization,
            });
        Json(json!({ "ok": true }))
    }

    let app = Router::new()
        .route("/provider", post(capture))
        .with_state(captured_for_handler);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local addr");
    tokio::spawn(async move {
        axum::serve(listener, app)
            .await
            .expect("fake provider server");
    });

    (format!("http://{addr}/provider"), captured)
}

async fn authorized_json_request(
    app: Router,
    method: &str,
    path: &str,
    body: Value,
    token: &str,
) -> (axum::http::StatusCode, Value) {
    let request = axum::http::Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::from(body.to_string()))
        .expect("valid request");
    let response = app.oneshot(request).await.expect("response");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

async fn public_json_request(
    app: Router,
    method: &str,
    path: &str,
    body: Value,
) -> (axum::http::StatusCode, Value) {
    let request = axum::http::Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .body(axum::body::Body::from(body.to_string()))
        .expect("valid request");
    let response = app.oneshot(request).await.expect("response");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let value = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, value)
}

#[tokio::test]
async fn otp_request_delivers_the_code_through_the_configured_sms_provider() {
    let (provider_url, captured) = spawn_fake_provider().await;

    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let mut state = AppState::memory();
    state.token_signing_secret = Some(String::from_utf8_lossy(secret).into_owned().into());
    state.sms_provider_url = Some(provider_url);
    state.sms_provider_bearer_token = Some("sms-provider-secret".into());

    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "Outbound Provider Test Tenant".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    state
        .tenants
        .create_tenant(&tenant, "outbound-provider-test")
        .await
        .expect("create tenant");
    let user = state
        .auth
        .register(
            "otp-provider-test@qervon.test".into(),
            "OTP Provider Test User".into(),
            "a-long-enough-test-password".into(),
            UserRole::Customer,
        )
        .await
        .expect("register user");
    state
        .tenants
        .add_member(&TenantMembership {
            tenant_id: tenant.id,
            user_id: user.id,
            role: TenantMemberRole::Member,
            joined_at: Utc::now(),
        })
        .await
        .expect("add tenant membership");
    let token = issue_access_token(
        secret,
        user.id.0,
        tenant.id.0,
        UserRole::Customer,
        ChronoDuration::minutes(5),
    )
    .expect("issue access token");

    let app = router(state);

    // Link the phone number to this signed-in account first — OtpService
    // resolves accounts strictly by phone, mirroring real OTP-login apps.
    let (status, _) = authorized_json_request(
        app.clone(),
        "POST",
        "/v1/auth/phone",
        json!({ "phone": "+905551234567" }),
        &token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, response) = public_json_request(
        app,
        "POST",
        "/v1/auth/otp/request",
        json!({ "tenant_slug": "outbound-provider-test", "phone": "+905551234567" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    // Postgres-mode behavior applies to the dev_code field regardless of
    // backend, since this is memory storage: the code is only echoed back
    // for local testing, never in a real deployment. What matters here is
    // that the provider actually received the SMS.
    assert!(response.get("status").is_some());

    let requests = captured.lock().expect("captured lock").clone();
    assert_eq!(
        requests.len(),
        1,
        "SMS provider should be called exactly once"
    );
    assert_eq!(requests[0].body["phone"], "+905551234567");
    assert!(requests[0].body["message"]
        .as_str()
        .expect("message field")
        .contains("Qervon OTP code:"));
    assert_eq!(
        requests[0].authorization,
        Some("Bearer sms-provider-secret".to_string())
    );
}

#[tokio::test]
async fn payment_charge_is_forwarded_to_the_configured_payment_gateway() {
    let (provider_url, captured) = spawn_fake_provider().await;

    let mut state = AppState::memory();
    state.api_access_token = Some("test-dev-token".into());
    state.payment_gateway_url = Some(provider_url);
    state.payment_gateway_bearer_token = Some("payment-gateway-secret".into());
    let app = router(state);

    let order_id = uuid::Uuid::now_v7();
    let (status, response) = authorized_json_request(
        app,
        "POST",
        "/v1/payments/charge",
        json!({
            "order_id": order_id,
            "amount_minor": 4500,
            "currency": "TRY",
            "method": "card"
        }),
        "test-dev-token",
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(response["status"], "accepted");

    let requests = captured.lock().expect("captured lock").clone();
    assert_eq!(
        requests.len(),
        1,
        "payment gateway should be called exactly once"
    );
    assert_eq!(requests[0].body["order_id"], order_id.to_string());
    assert_eq!(requests[0].body["amount_minor"], 4500);
    assert_eq!(requests[0].body["currency"], "TRY");
    assert_eq!(requests[0].body["method"], "card");
    assert_eq!(
        requests[0].authorization,
        Some("Bearer payment-gateway-secret".to_string())
    );
}

#[tokio::test]
async fn payment_charge_is_simulated_when_no_gateway_is_configured() {
    // No provider URL set at all: this must never fail or block the
    // caller, and must clearly say "simulated" rather than pretending
    // real money moved.
    let mut state = AppState::memory();
    state.api_access_token = Some("test-dev-token".into());
    let app = router(state);

    let (status, response) = authorized_json_request(
        app,
        "POST",
        "/v1/payments/charge",
        json!({
            "order_id": uuid::Uuid::now_v7(),
            "amount_minor": 1200,
            "currency": "TRY",
            "method": "qr"
        }),
        "test-dev-token",
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(response["status"], "simulated");
}

#[tokio::test]
async fn native_push_dispatch_is_forwarded_to_the_configured_push_provider() {
    let (provider_url, captured) = spawn_fake_provider().await;

    let mut state = AppState::memory();
    state.api_access_token = Some("test-dev-token".into());
    state.push_provider_url = Some(provider_url);
    state.push_provider_bearer_token = Some("push-provider-secret".into());

    let user_id = UserId::new();
    state
        .device_push
        .register(
            user_id,
            PushPlatform::Ios,
            qervon_domain::AppVariant::Courier,
            "device-token-abc".into(),
        )
        .await
        .expect("register device token");

    let app = router(state);
    let (status, response) = authorized_json_request(
        app,
        "POST",
        "/v1/push/native/dispatch",
        json!({
            "user_id": user_id.0,
            "platform": "ios",
            "title": "Kuryeniz yolda",
            "body": "Siparişiniz 10 dakika içinde teslim edilecek"
        }),
        "test-dev-token",
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(response["status"], "sent");

    let requests = captured.lock().expect("captured lock").clone();
    assert_eq!(
        requests.len(),
        1,
        "push provider should be called exactly once"
    );
    assert_eq!(requests[0].body["user_id"], user_id.0.to_string());
    assert_eq!(requests[0].body["tokens"][0], "device-token-abc");
    assert_eq!(
        requests[0].authorization,
        Some("Bearer push-provider-secret".to_string())
    );
}

#[tokio::test]
async fn native_push_dispatch_is_skipped_when_the_user_has_no_registered_devices() {
    let mut state = AppState::memory();
    state.api_access_token = Some("test-dev-token".into());
    // No provider configured either, but the point of this test is the
    // no-device-tokens short-circuit happening before any provider call.
    let app = router(state);

    let (status, response) = authorized_json_request(
        app,
        "POST",
        "/v1/push/native/dispatch",
        json!({
            "user_id": UserId::new().0,
            "platform": "android",
            "title": "Test",
            "body": "Test body"
        }),
        "test-dev-token",
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(response["status"], "skipped");
    assert_eq!(response["reason"], "no_device_tokens");
}
