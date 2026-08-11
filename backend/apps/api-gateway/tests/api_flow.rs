// =============================================================================
// File:           backend/apps/api-gateway/tests/api_flow.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   HTTP-level end-to-end test of the delivery vertical slice.
//
// Specification:
//   QAS-000002, QAS-000003, QAS-000005, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use axum::Router;
use chrono::{Duration, Utc};
use http_body_util::BodyExt;
use qervon_api_gateway::auth::issue_access_token;
use qervon_api_gateway::http::router;
use qervon_api_gateway::state::AppState;
use qervon_domain::{
    OrderId, ProofOfDeliveryRepository, TenantCompany, TenantId, TenantMemberRole,
    TenantMembership, TenantRepository, UserRole,
};
use serde_json::{json, Value};
use tower::ServiceExt;
use uuid::Uuid;

fn app() -> Router {
    let mut state = AppState::memory();
    state.api_access_token = Some("test-dev-token".into());
    router(state)
}

fn protected_app() -> Router {
    let mut state = AppState::memory();
    state.api_access_token = Some("test-access-token".into());
    router(state)
}

fn signed_token_app() -> Router {
    let mut state = AppState::memory();
    state.token_signing_secret = Some("test-signing-secret-must-be-at-least-32-characters".into());
    router(state)
}

async fn request(
    app: Router,
    method: &str,
    path: &str,
    body: Value,
) -> (axum::http::StatusCode, Value) {
    let request = axum::http::Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .header("authorization", "Bearer test-dev-token")
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

async fn unauthenticated_request(
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

async fn authorized_request(
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

async fn tenant_auth_app() -> Router {
    let mut state = AppState::memory();
    state.token_signing_secret = Some("test-signing-secret-must-be-at-least-32-characters".into());
    let user = state
        .auth
        .register(
            "operator@qervon.test".into(),
            "Qervon Operator".into(),
            "a-long-enough-test-password".into(),
            UserRole::Operator,
        )
        .await
        .expect("create test user");
    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "Qervon Test Logistics".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    state
        .tenants
        .create_tenant(&tenant, "qervon-test")
        .await
        .expect("create tenant");
    state
        .tenants
        .add_member(&TenantMembership {
            tenant_id: tenant.id,
            user_id: user.id,
            role: TenantMemberRole::Operator,
            joined_at: Utc::now(),
        })
        .await
        .expect("add tenant membership");
    router(state)
}

async fn tenant_tracking_fixture() -> (Router, String, String) {
    let mut state = AppState::memory();
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    state.token_signing_secret = Some(String::from_utf8_lossy(secret).into_owned().into());
    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "Tracking Tenant".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    state
        .tenants
        .create_tenant(&tenant, "tracking-tenant")
        .await
        .expect("create tracking tenant");
    let other_tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "Other Tenant".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    state
        .tenants
        .create_tenant(&other_tenant, "other-tenant")
        .await
        .expect("create other tenant");
    let operator_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        tenant.id.0,
        UserRole::Dispatcher,
        Duration::minutes(5),
    )
    .expect("operator token");
    let other_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        other_tenant.id.0,
        UserRole::Dispatcher,
        Duration::minutes(5),
    )
    .expect("other token");
    (router(state), operator_token, other_token)
}

#[tokio::test]
async fn protected_api_rejects_missing_or_invalid_bearer_token() {
    let (status, body) = unauthenticated_request(
        protected_app(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Ali", "vehicle": "motorcycle" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNAUTHORIZED);
    assert_eq!(body["status"], 401);

    let (status, _) = authorized_request(
        protected_app(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Ali", "vehicle": "motorcycle" }),
        "wrong-token",
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn unconfigured_api_fails_closed() {
    let (status, body) = unauthenticated_request(
        router(AppState::memory()),
        "POST",
        "/v1/couriers",
        json!({ "name": "Ali", "vehicle": "motorcycle" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["status"], 503);
}

#[tokio::test]
async fn protected_api_accepts_valid_bearer_token() {
    let (status, courier) = authorized_request(
        protected_app(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Ali", "vehicle": "motorcycle" }),
        "test-access-token",
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(courier["name"], "Ali");
}

#[tokio::test]
async fn protected_api_accepts_valid_signed_user_token() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let token = issue_access_token(
        secret,
        Uuid::now_v7(),
        Uuid::now_v7(),
        UserRole::Dispatcher,
        Duration::minutes(5),
    )
    .expect("issue signed token");
    let (status, courier) = authorized_request(
        signed_token_app(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Ayşe", "vehicle": "motorcycle" }),
        &token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(courier["name"], "Ayşe");
}

#[tokio::test]
async fn customer_token_cannot_operate_courier_or_order_routes() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let token = issue_access_token(
        secret,
        Uuid::now_v7(),
        Uuid::now_v7(),
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("issue customer token");
    let (status, body) = authorized_request(
        signed_token_app(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Yetkisiz", "vehicle": "motorcycle" }),
        &token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
    assert_eq!(body["status"], 403);
}

#[tokio::test]
async fn signed_users_are_routed_to_tenant_scoped_live_tracking_stream() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let token = issue_access_token(
        secret,
        Uuid::now_v7(),
        Uuid::now_v7(),
        UserRole::Dispatcher,
        Duration::minutes(5),
    )
    .expect("issue dispatcher token");
    let (status, body) =
        authorized_request(signed_token_app(), "GET", "/ws/tracking", json!({}), &token).await;
    // Authentication and tenant scoping have passed; this plain HTTP request
    // intentionally lacks the WebSocket upgrade headers.
    assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
    assert!(body.is_null());
}

#[tokio::test]
async fn order_lifecycle_over_http() {
    let app = app();

    let (status, courier) = request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Ali Kurye", "vehicle": "motorcycle" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = courier["id"].as_str().expect("courier id");

    let (status, _) = request(
        app.clone(),
        "POST",
        &format!("/v1/couriers/{courier_id}/location"),
        json!({ "latitude": 41.0, "longitude": 29.0 }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, order) = request(
        app.clone(),
        "POST",
        "/v1/orders",
        json!({
            "customer_id": "00000000-0000-7000-8000-000000000001",
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "pickup" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "dropoff" },
            "fare_amount_minor": 1500,
            "fare_currency": "TRY"
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order_id = order["id"].as_str().expect("order id");

    let (status, _) = request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/assign"),
        json!({ "courier_id": courier_id }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, transit) = request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/transit"),
        json!({ "recipient_name": "Teslim Alan", "qr_barcode_verified": true }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(transit["status"], "in_transit");

    let (status, delivered) = request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/deliver"),
        json!({ "recipient_name": "Teslim Alan", "qr_barcode_verified": true }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(delivered["status"], "delivered");

    let (status, fetched) = request(app, "GET", &format!("/v1/orders/{order_id}"), json!({})).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(fetched["status"], "delivered");
}

#[tokio::test]
async fn rejecting_invalid_vehicle_returns_422() {
    let app = app();
    let (status, body) = request(
        app,
        "POST",
        "/v1/couriers",
        json!({ "name": "Ali", "vehicle": "rocket" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(body["status"], 422);
}

#[tokio::test]
async fn register_user_over_http() {
    let app = app();
    let (status, user) = request(
        app,
        "POST",
        "/v1/users",
        json!({
            "email": "ahmet@qervon.com",
            "display_name": "Ahmet Yılmaz",
            "role": "admin"
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(user["email"], "ahmet@qervon.com");
    assert_eq!(user["role"], "admin");
    assert_eq!(user["status"], "active");
}

#[tokio::test]
async fn public_registration_only_creates_a_customer() {
    let mut state = AppState::memory();
    state.api_access_token = Some("test-dev-token".into());
    let (status, _) = request(
        router(state.clone()),
        "POST",
        "/v1/auth/register",
        json!({
            "email": "customer@qervon.test",
            "display_name": "Customer",
            "password": "a-long-enough-test-password",
            "role": "super_admin"
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let user = state
        .identity
        .get_user_by_email("customer@qervon.test")
        .await
        .expect("registered customer");
    assert_eq!(user.role, UserRole::Customer);
}

#[tokio::test]
async fn login_refresh_and_logout_require_a_real_tenant_membership() {
    let app = tenant_auth_app().await;
    let (status, login) = request(
        app.clone(),
        "POST",
        "/v1/auth/login",
        json!({
            "email": "operator@qervon.test",
            "password": "a-long-enough-test-password",
            "tenant_slug": "qervon-test"
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(login["token_type"], "Bearer");
    let refresh = login["refresh_token"]
        .as_str()
        .expect("refresh token")
        .to_owned();

    let (status, rotated) = request(
        app.clone(),
        "POST",
        "/v1/auth/refresh",
        json!({ "refresh_token": refresh }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let rotated_refresh = rotated["refresh_token"]
        .as_str()
        .expect("rotated refresh token")
        .to_owned();

    let (status, _) = request(
        app.clone(),
        "POST",
        "/v1/auth/refresh",
        json!({ "refresh_token": refresh }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNAUTHORIZED);

    let (status, _) = request(
        app.clone(),
        "POST",
        "/v1/auth/logout",
        json!({ "refresh_token": rotated_refresh }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NO_CONTENT);

    let (status, _) = request(
        app,
        "POST",
        "/v1/auth/refresh",
        json!({ "refresh_token": rotated_refresh }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn live_tracking_isolated_by_tenant_for_fleet_and_order() {
    let (app, token, other_tenant_token) = tenant_tracking_fixture().await;
    let (status, courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Tenant Kurye", "vehicle": "motorcycle" }),
        &token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = courier["id"].as_str().expect("courier id");

    let customer_id = Uuid::now_v7();
    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/orders",
        json!({
            "customer_id": customer_id,
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "pickup" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "dropoff" },
            "fare_amount_minor": 1500,
            "fare_currency": "TRY"
        }),
        &token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order_id = order["id"].as_str().expect("order id");

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/assign"),
        json!({ "courier_id": courier_id }),
        &token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/couriers/{courier_id}/location"),
        json!({ "latitude": 41.02, "longitude": 29.02, "speed_kmh": 21.5, "battery_pct": 87 }),
        &token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, locations) =
        authorized_request(app.clone(), "GET", "/v1/tracking/live", json!({}), &token).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(locations.as_array().expect("location array").len(), 1);
    assert_eq!(locations[0]["courier_id"], courier_id);

    let (status, tracked) = authorized_request(
        app.clone(),
        "GET",
        &format!("/v1/orders/{order_id}/tracking"),
        json!({ "recipient_name": "Teslim Alan", "qr_barcode_verified": true }),
        &token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(tracked["courier_id"], courier_id);

    let (status, other_locations) = authorized_request(
        app.clone(),
        "GET",
        "/v1/tracking/live",
        json!({}),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(other_locations
        .as_array()
        .expect("location array")
        .is_empty());

    let (status, _) = authorized_request(
        app,
        "GET",
        &format!("/v1/orders/{order_id}/tracking"),
        json!({}),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn browser_login_uses_http_only_cookies_and_rejects_missing_csrf() {
    let mut state = AppState::memory();
    state.token_signing_secret = Some("test-signing-secret-must-be-at-least-32-characters".into());
    let user = state
        .auth
        .register(
            "browser@qervon.test".into(),
            "Browser User".into(),
            "a-long-enough-test-password".into(),
            UserRole::Operator,
        )
        .await
        .expect("user");
    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "Browser Tenant".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    state
        .tenants
        .create_tenant(&tenant, "browser-tenant")
        .await
        .expect("tenant");
    state
        .tenants
        .add_member(&TenantMembership {
            tenant_id: tenant.id,
            user_id: user.id,
            role: TenantMemberRole::Operator,
            joined_at: Utc::now(),
        })
        .await
        .expect("membership");
    let app = router(state);
    let request = axum::http::Request::builder().method("POST").uri("/v1/browser/auth/login").header("content-type", "application/json").body(axum::body::Body::from(json!({"email":"browser@qervon.test","password":"a-long-enough-test-password","tenant_slug":"browser-tenant"}).to_string())).expect("request");
    let response = app.clone().oneshot(request).await.expect("response");
    assert_eq!(response.status(), axum::http::StatusCode::NO_CONTENT);
    let cookies: Vec<_> = response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|v| v.to_str().unwrap())
        .collect();
    assert!(cookies
        .iter()
        .any(|cookie| cookie.starts_with("qervon_access_token=") && cookie.contains("HttpOnly")));
    assert!(cookies
        .iter()
        .any(|cookie| cookie.starts_with("qervon_csrf_token=")));
    let access = cookies
        .iter()
        .find(|cookie| cookie.starts_with("qervon_access_token="))
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let csrf = cookies
        .iter()
        .find(|cookie| cookie.starts_with("qervon_csrf_token="))
        .unwrap()
        .split(';')
        .next()
        .unwrap();
    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/v1/couriers")
        .header("content-type", "application/json")
        .header("cookie", format!("{access}; {csrf}"))
        .body(axum::body::Body::from(
            json!({"name":"No CSRF","vehicle":"motorcycle"}).to_string(),
        ))
        .expect("request");
    let response = app.oneshot(request).await.expect("response");
    assert_eq!(response.status(), axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn auto_dispatch_never_assigns_a_courier_from_another_tenant() {
    let (app, tenant_token, other_tenant_token) = tenant_tracking_fixture().await;

    let (status, foreign_courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Foreign Courier", "vehicle": "motorcycle" }),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let foreign_courier_id = foreign_courier["id"].as_str().expect("foreign courier id");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/couriers/{foreign_courier_id}/location"),
        json!({ "latitude": 41.0000, "longitude": 29.0000 }),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, tenant_courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Tenant Courier", "vehicle": "motorcycle" }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let tenant_courier_id = tenant_courier["id"].as_str().expect("tenant courier id");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/couriers/{tenant_courier_id}/location"),
        json!({ "latitude": 41.0100, "longitude": 29.0100 }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/orders",
        json!({
            "customer_id": Uuid::now_v7(),
            "pickup": { "latitude": 41.0001, "longitude": 29.0001, "label": "Pickup" },
            "dropoff": { "latitude": 41.1000, "longitude": 29.1000, "label": "Dropoff" },
            "fare_amount_minor": 1500,
            "fare_currency": "TRY"
        }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order_id = order["id"].as_str().expect("order id");

    let (status, assignment) = authorized_request(
        app,
        "POST",
        &format!("/v1/orders/{order_id}/assign"),
        json!({ "courier_id": null }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(assignment["courier_id"], tenant_courier_id);
    assert_ne!(assignment["courier_id"], foreign_courier_id);
}

#[tokio::test]
async fn browser_cookie_session_rotates_refresh_and_allows_valid_csrf_requests() {
    let mut state = AppState::memory();
    state.token_signing_secret = Some("test-signing-secret-must-be-at-least-32-characters".into());
    let user = state
        .auth
        .register(
            "session@qervon.test".into(),
            "Session User".into(),
            "a-long-enough-test-password".into(),
            UserRole::Operator,
        )
        .await
        .expect("user");
    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "Session Tenant".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    state
        .tenants
        .create_tenant(&tenant, "session-tenant")
        .await
        .expect("tenant");
    state
        .tenants
        .add_member(&TenantMembership {
            tenant_id: tenant.id,
            user_id: user.id,
            role: TenantMemberRole::Operator,
            joined_at: Utc::now(),
        })
        .await
        .expect("membership");
    let app = router(state);

    let login = axum::http::Request::builder()
        .method("POST")
        .uri("/v1/browser/auth/login")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            json!({"email":"session@qervon.test","password":"a-long-enough-test-password","tenant_slug":"session-tenant"}).to_string(),
        ))
        .expect("login request");
    let login_response = app.clone().oneshot(login).await.expect("login response");
    assert_eq!(login_response.status(), axum::http::StatusCode::NO_CONTENT);
    let cookies: Vec<String> = login_response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|value| value.to_str().expect("set-cookie").to_owned())
        .collect();
    let access = cookies
        .iter()
        .find(|cookie| cookie.starts_with("qervon_access_token="))
        .expect("access cookie")
        .split(';')
        .next()
        .expect("access pair")
        .to_owned();
    let refresh = cookies
        .iter()
        .find(|cookie| cookie.starts_with("qervon_refresh_token="))
        .expect("refresh cookie")
        .split(';')
        .next()
        .expect("refresh pair")
        .to_owned();
    let csrf = cookies
        .iter()
        .find(|cookie| cookie.starts_with("qervon_csrf_token="))
        .expect("csrf cookie")
        .split(';')
        .next()
        .expect("csrf pair")
        .to_owned();
    let csrf_value = csrf.split_once('=').expect("csrf value").1.to_owned();

    let create = axum::http::Request::builder()
        .method("POST")
        .uri("/v1/couriers")
        .header("content-type", "application/json")
        .header("cookie", format!("{access}; {csrf}"))
        .header("x-csrf-token", &csrf_value)
        .body(axum::body::Body::from(
            json!({"name":"CSRF Courier","vehicle":"motorcycle"}).to_string(),
        ))
        .expect("create request");
    assert_eq!(
        app.clone()
            .oneshot(create)
            .await
            .expect("create response")
            .status(),
        axum::http::StatusCode::CREATED
    );

    let refresh_request = axum::http::Request::builder()
        .method("POST")
        .uri("/v1/browser/auth/refresh")
        .header("cookie", format!("{refresh}; {csrf}"))
        .header("x-csrf-token", &csrf_value)
        .body(axum::body::Body::empty())
        .expect("refresh request");
    let refresh_response = app
        .clone()
        .oneshot(refresh_request)
        .await
        .expect("refresh response");
    assert_eq!(
        refresh_response.status(),
        axum::http::StatusCode::NO_CONTENT
    );
    let next_cookies: Vec<String> = refresh_response
        .headers()
        .get_all("set-cookie")
        .iter()
        .map(|value| value.to_str().expect("set-cookie").to_owned())
        .collect();
    let next_refresh = next_cookies
        .iter()
        .find(|cookie| cookie.starts_with("qervon_refresh_token="))
        .expect("next refresh cookie")
        .split(';')
        .next()
        .expect("next refresh pair")
        .to_owned();
    let next_csrf = next_cookies
        .iter()
        .find(|cookie| cookie.starts_with("qervon_csrf_token="))
        .expect("next csrf cookie")
        .split(';')
        .next()
        .expect("next csrf pair")
        .to_owned();
    let next_csrf_value = next_csrf
        .split_once('=')
        .expect("next csrf value")
        .1
        .to_owned();
    assert_ne!(refresh, next_refresh);

    let reused_refresh = axum::http::Request::builder()
        .method("POST")
        .uri("/v1/browser/auth/refresh")
        .header("cookie", format!("{refresh}; {csrf}"))
        .header("x-csrf-token", &csrf_value)
        .body(axum::body::Body::empty())
        .expect("reuse request");
    assert_eq!(
        app.clone()
            .oneshot(reused_refresh)
            .await
            .expect("reuse response")
            .status(),
        axum::http::StatusCode::UNAUTHORIZED
    );

    let logout = axum::http::Request::builder()
        .method("POST")
        .uri("/v1/browser/auth/logout")
        .header("cookie", format!("{next_refresh}; {next_csrf}"))
        .header("x-csrf-token", &next_csrf_value)
        .body(axum::body::Body::empty())
        .expect("logout request");
    let logout_response = app.oneshot(logout).await.expect("logout response");
    assert_eq!(logout_response.status(), axum::http::StatusCode::NO_CONTENT);
    assert_eq!(
        logout_response
            .headers()
            .get_all("set-cookie")
            .iter()
            .count(),
        3
    );
    assert!(logout_response
        .headers()
        .get_all("set-cookie")
        .iter()
        .all(|cookie| cookie.to_str().expect("set-cookie").contains("Max-Age=0")));
}

#[tokio::test]
async fn courier_can_only_manage_its_own_assigned_workflow() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "Courier Workflow Tenant".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    let mut state = AppState::memory();
    state.token_signing_secret = Some(String::from_utf8_lossy(secret).into_owned().into());
    let proofs_of_delivery = state.proofs_of_delivery.clone();
    state
        .tenants
        .create_tenant(&tenant, "courier-workflow")
        .await
        .expect("tenant");
    let dispatcher_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        tenant.id.0,
        UserRole::Dispatcher,
        Duration::minutes(5),
    )
    .expect("dispatcher token");
    let courier_id = Uuid::now_v7();
    let other_courier_id = Uuid::now_v7();
    let courier_token = issue_access_token(
        secret,
        courier_id,
        tenant.id.0,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier token");
    let other_courier_token = issue_access_token(
        secret,
        other_courier_id,
        tenant.id.0,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("other courier token");
    let app = router(state);

    for (id, name) in [
        (courier_id, "Assigned Courier"),
        (other_courier_id, "Other Courier"),
    ] {
        let (status, _) = authorized_request(
            app.clone(),
            "POST",
            "/v1/couriers",
            json!({ "id": id, "name": name, "vehicle": "motorcycle" }),
            &dispatcher_token,
        )
        .await;
        assert_eq!(status, axum::http::StatusCode::CREATED);
    }

    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/orders",
        json!({
            "customer_id": Uuid::now_v7(),
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Pickup" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Dropoff" },
            "fare_amount_minor": 1500,
            "fare_currency": "TRY"
        }),
        &dispatcher_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order_id = order["id"].as_str().expect("order id");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/assign"),
        json!({ "courier_id": courier_id }),
        &dispatcher_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, fleet) = authorized_request(
        app.clone(),
        "GET",
        "/v1/couriers",
        json!({}),
        &dispatcher_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(fleet.as_array().expect("fleet list").len(), 2);
    assert!(fleet
        .as_array()
        .expect("fleet list")
        .iter()
        .any(|courier| courier["id"] == courier_id.to_string() && courier["status"] == "busy"));

    let (status, work) = authorized_request(
        app.clone(),
        "GET",
        "/v1/courier/orders",
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(work.as_array().expect("work list").len(), 1);
    assert_eq!(work[0]["id"], order_id);

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/deliver"),
        json!({ "recipient_name": "Teslim Alan", "qr_barcode_verified": true }),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CONFLICT);

    let (status, picked_up) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/pickup"),
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(picked_up["status"], "in_transit");

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/deliver"),
        json!({ "recipient_name": "Teslim Alan", "qr_barcode_verified": true }),
        &other_courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);

    let (status, delivered) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/deliver"),
        json!({ "recipient_name": "Teslim Alan", "qr_barcode_verified": true }),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(delivered["status"], "delivered");
    let persisted_proof = proofs_of_delivery
        .find_by_order(OrderId(order_id.parse().expect("valid order id")))
        .await
        .expect("find proof")
        .expect("persisted proof");
    assert_eq!(persisted_proof.recipient_name, "Teslim Alan");
    assert!(persisted_proof.qr_barcode_verified);

    let (status, offline) = authorized_request(
        app.clone(),
        "POST",
        "/v1/courier/me/status",
        json!({ "online": false }),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(offline["status"], "offline");
    let (status, online) = authorized_request(
        app,
        "POST",
        "/v1/courier/me/status",
        json!({ "online": true }),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(online["status"], "available");
}

#[tokio::test]
async fn customer_orders_are_owned_by_session_and_admin_overview_is_tenant_scoped() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "Customer Workflow Tenant".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    let mut state = AppState::memory();
    state.token_signing_secret = Some(String::from_utf8_lossy(secret).into_owned().into());
    state
        .tenants
        .create_tenant(&tenant, "customer-workflow")
        .await
        .expect("tenant");
    let customer_id = Uuid::now_v7();
    let customer_token = issue_access_token(
        secret,
        customer_id,
        tenant.id.0,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("customer token");
    let other_customer_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        tenant.id.0,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("other customer token");
    let operator_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        tenant.id.0,
        UserRole::Operator,
        Duration::minutes(5),
    )
    .expect("operator token");
    let app = router(state);

    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/orders",
        json!({
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Müşteri alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Müşteri teslim" },
            "fare_amount_minor": 4500,
            "fare_currency": "TRY"
        }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(order["customer_id"], customer_id.to_string());

    let (status, mine) = authorized_request(
        app.clone(),
        "GET",
        "/v1/customer/orders",
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(mine.as_array().expect("customer orders").len(), 1);

    let (status, other) = authorized_request(
        app.clone(),
        "GET",
        "/v1/customer/orders",
        json!({}),
        &other_customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(other.as_array().expect("other customer orders").is_empty());

    let (status, overview) = authorized_request(
        app.clone(),
        "GET",
        "/v1/operations/overview",
        json!({}),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(overview["active_orders"], 1);
    assert_eq!(overview["pending_orders"], 1);

    let (status, _) = authorized_request(
        app,
        "GET",
        "/v1/operations/overview",
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn runtime_endpoints_expose_safe_observability_contracts() {
    let app = app();
    let response = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/health")
                .body(axum::body::Body::empty())
                .expect("health request"),
        )
        .await
        .expect("health response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    assert!(response.headers().contains_key("x-request-id"));
    assert_eq!(response.headers()["x-content-type-options"], "nosniff");
    assert_eq!(response.headers()["referrer-policy"], "no-referrer");

    let ready = app
        .clone()
        .oneshot(
            axum::http::Request::builder()
                .uri("/ready")
                .body(axum::body::Body::empty())
                .expect("ready request"),
        )
        .await
        .expect("ready response");
    assert_eq!(ready.status(), axum::http::StatusCode::OK);

    let metrics = app
        .oneshot(
            axum::http::Request::builder()
                .uri("/metrics")
                .body(axum::body::Body::empty())
                .expect("metrics request"),
        )
        .await
        .expect("metrics response");
    assert_eq!(metrics.status(), axum::http::StatusCode::OK);
    let body = metrics
        .into_body()
        .collect()
        .await
        .expect("metrics body")
        .to_bytes();
    let metrics = String::from_utf8(body.to_vec()).expect("UTF-8 metrics");
    assert!(metrics.contains("qervon_http_requests_total{status_class=\"2xx\"}"));
    assert!(metrics.contains("qervon_process_uptime_seconds"));

    let not_ready = router(AppState::memory())
        .oneshot(
            axum::http::Request::builder()
                .uri("/ready")
                .body(axum::body::Body::empty())
                .expect("unconfigured ready request"),
        )
        .await
        .expect("unconfigured ready response");
    assert_eq!(
        not_ready.status(),
        axum::http::StatusCode::SERVICE_UNAVAILABLE
    );
}

#[tokio::test]
async fn oversized_json_request_is_rejected() {
    let response = app()
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri("/v1/auth/login")
                .header("content-type", "application/json")
                .body(axum::body::Body::from(vec![b'a'; 1_048_577]))
                .expect("oversized request"),
        )
        .await
        .expect("oversized response");
    assert_eq!(response.status(), axum::http::StatusCode::PAYLOAD_TOO_LARGE);
}
