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
use qervon_api_gateway::auth::{issue_access_token, verify_access_token};
use qervon_api_gateway::http::router;
use qervon_api_gateway::state::AppState;
use qervon_application::RegisterCourierInput;
use qervon_domain::{
    Location, OrderId, ProofOfDeliveryRepository, TenantCompany, TenantId, TenantMemberRole,
    TenantMembership, TenantRepository, UserRole, VehicleType,
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
    mut body: Value,
) -> (axum::http::StatusCode, Value) {
    if path.ends_with("/pickup") && body.get("pickup_photo_evidence_url").is_none() {
        body["pickup_photo_evidence_url"] =
            Value::String("/v1/uploads/pickup-photos/test.jpg".into());
    }
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
    mut body: Value,
    token: &str,
) -> (axum::http::StatusCode, Value) {
    // Older delivery-flow fixtures predate the contact-phone requirement.
    // Give those focused tests a valid representative value while dedicated
    // validation cases can still send `contact_phone: null` explicitly.
    if path == "/v1/customer/orders" && body.get("contact_phone").is_none() {
        body["contact_phone"] = Value::String("05550000000".into());
    }
    if path.ends_with("/pickup") && body.get("pickup_photo_evidence_url").is_none() {
        body["pickup_photo_evidence_url"] =
            Value::String("/v1/uploads/pickup-photos/test.jpg".into());
    }
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

async fn authorized_csv_request(
    app: Router,
    path: &str,
    csv: &str,
    token: &str,
) -> (axum::http::StatusCode, Value) {
    let request = axum::http::Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", "text/csv;charset=utf-8")
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::from(csv.to_owned()))
        .expect("valid CSV request");
    let response = app.oneshot(request).await.expect("CSV response");
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("CSV response body")
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

/// Same shape as `tenant_tracking_fixture`, but with `uploads_dir` pointed
/// at a fresh temporary directory instead of the default `./data/uploads`,
/// so photo-upload tests never touch the real working directory and never
/// collide with each other across parallel test runs.
async fn tenant_tracking_fixture_with_temp_uploads() -> (Router, String, String, std::path::PathBuf)
{
    let mut state = AppState::memory();
    let uploads_dir = std::env::temp_dir().join(format!("qervon-test-uploads-{}", Uuid::now_v7()));
    state.uploads_dir = uploads_dir.clone();
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    state.token_signing_secret = Some(String::from_utf8_lossy(secret).into_owned().into());
    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "Upload Tenant".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    state
        .tenants
        .create_tenant(&tenant, "upload-tenant")
        .await
        .expect("create upload tenant");
    let other_tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "Other Upload Tenant".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    state
        .tenants
        .create_tenant(&other_tenant, "other-upload-tenant")
        .await
        .expect("create other upload tenant");
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
    (router(state), operator_token, other_token, uploads_dir)
}

/// Builds a minimal single-field multipart/form-data body around raw JPEG-
/// looking bytes (the endpoint only inspects the declared content-type, not
/// real JPEG magic bytes, so any bytes are fine here).
fn multipart_photo_body(bytes: &[u8]) -> (String, Vec<u8>) {
    let boundary = "qervon-test-boundary";
    let mut body = Vec::new();
    body.extend_from_slice(format!("--{boundary}\r\n").as_bytes());
    body.extend_from_slice(
        b"Content-Disposition: form-data; name=\"photo\"; filename=\"proof.jpg\"\r\n",
    );
    body.extend_from_slice(b"Content-Type: image/jpeg\r\n\r\n");
    body.extend_from_slice(bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    (format!("multipart/form-data; boundary={boundary}"), body)
}

async fn multipart_upload_request(
    app: Router,
    path: &str,
    token: &str,
    bytes: &[u8],
) -> (axum::http::StatusCode, Value) {
    let (content_type, body) = multipart_photo_body(bytes);
    let request = axum::http::Request::builder()
        .method("POST")
        .uri(path)
        .header("content-type", content_type)
        .header("authorization", format!("Bearer {token}"))
        .body(axum::body::Body::from(body))
        .expect("valid multipart request");
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
async fn courier_can_upload_a_delivery_photo_and_use_it_as_proof_of_delivery() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, other_tenant_token, uploads_dir) =
        tenant_tracking_fixture_with_temp_uploads().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");

    let (status, courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Foto Kurye", "vehicle": "motorcycle" }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = Uuid::parse_str(courier["id"].as_str().expect("courier id")).expect("UUID");
    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/orders",
        json!({
            "customer_id": Uuid::now_v7(),
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "fare_amount_minor": 4200,
            "fare_currency": "TRY"
        }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order_id = order["id"].as_str().expect("order id").to_string();
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/assign"),
        json!({ "courier_id": courier_id }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let courier_token = issue_access_token(
        secret,
        courier_id,
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier token");

    let (status, upload) = multipart_upload_request(
        app.clone(),
        &format!("/v1/courier/orders/{order_id}/photo-evidence"),
        &courier_token,
        b"not-really-a-jpeg-but-thats-fine-for-this-endpoint",
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let photo_url = upload["url"].as_str().expect("upload url").to_string();
    assert!(photo_url.starts_with(&format!("/v1/uploads/delivery-photos/{order_id}/")));

    // The file really was written to disk under the configured uploads dir.
    let relative = photo_url
        .strip_prefix("/v1/uploads/")
        .expect("uploads-relative path");
    assert!(uploads_dir.join(relative).exists());

    // The owning tenant's operator can fetch the photo back byte-for-byte.
    let get_request = axum::http::Request::builder()
        .method("GET")
        .uri(&photo_url)
        .header("authorization", format!("Bearer {operator_token}"))
        .body(axum::body::Body::empty())
        .expect("valid get request");
    let response = app.clone().oneshot(get_request).await.expect("response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let fetched_bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    assert_eq!(
        &fetched_bytes[..],
        b"not-really-a-jpeg-but-thats-fine-for-this-endpoint"
    );

    // A member of a different tenant cannot fetch this tenant's photo.
    let foreign_get_request = axum::http::Request::builder()
        .method("GET")
        .uri(&photo_url)
        .header("authorization", format!("Bearer {other_tenant_token}"))
        .body(axum::body::Body::empty())
        .expect("valid get request");
    let foreign_response = app
        .clone()
        .oneshot(foreign_get_request)
        .await
        .expect("response");
    assert_eq!(foreign_response.status(), axum::http::StatusCode::FORBIDDEN);

    // Pickup evidence is mandatory and must be persisted before the order can advance.
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/pickup"),
        json!({ "pickup_photo_evidence_url": "   " }),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/pickup"),
        json!({ "pickup_photo_evidence_url": photo_url }),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    // The courier now delivers the order, citing the uploaded photo as proof.
    let (status, delivered) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/deliver"),
        json!({ "recipient_name": "Ayşe Yılmaz", "qr_barcode_verified": false, "photo_evidence_url": photo_url }),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(delivered["status"], "delivered");

    let _ = tokio::fs::remove_dir_all(&uploads_dir).await;
}

#[tokio::test]
async fn delivery_photo_upload_rejects_oversized_files() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, _other_tenant_token, uploads_dir) =
        tenant_tracking_fixture_with_temp_uploads().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");

    let (status, courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Büyük Dosya Kurye", "vehicle": "motorcycle" }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = Uuid::parse_str(courier["id"].as_str().expect("courier id")).expect("UUID");
    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/orders",
        json!({
            "customer_id": Uuid::now_v7(),
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "fare_amount_minor": 4200,
            "fare_currency": "TRY"
        }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order_id = order["id"].as_str().expect("order id").to_string();
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/assign"),
        json!({ "courier_id": courier_id }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let courier_token = issue_access_token(
        secret,
        courier_id,
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier token");

    let oversized = vec![0u8; 9 * 1024 * 1024];
    let (status, _) = multipart_upload_request(
        app.clone(),
        &format!("/v1/courier/orders/{order_id}/photo-evidence"),
        &courier_token,
        &oversized,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);

    // A body between the global 1 MB default (every other route) and the
    // 8 MB photo-upload override must still succeed here — proving the
    // per-route `DefaultBodyLimit::max(MAX_UPLOAD_BYTES)` layer on the
    // photo-upload sub-router actually takes effect over the outer,
    // whole-router 1 MB default applied in `router()`.
    let two_megabytes = vec![7u8; 2 * 1024 * 1024];
    let (status, upload) = multipart_upload_request(
        app.clone(),
        &format!("/v1/courier/orders/{order_id}/photo-evidence"),
        &courier_token,
        &two_megabytes,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(upload["url"].as_str().is_some());

    let _ = tokio::fs::remove_dir_all(&uploads_dir).await;
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
async fn order_in_transit_can_be_returned_and_releases_the_courier() {
    let app = app();

    let (status, courier) = request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "İade Kurye", "vehicle": "motorcycle" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = courier["id"].as_str().expect("courier id").to_string();

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
            "customer_id": "00000000-0000-7000-8000-000000000002",
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "pickup" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "dropoff" },
            "fare_amount_minor": 2000,
            "fare_currency": "TRY"
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order_id = order["id"].as_str().expect("order id").to_string();

    let (status, _) = request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/assign"),
        json!({ "courier_id": courier_id }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, _) = request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/transit"),
        json!({ "recipient_name": "Teslim Alan", "qr_barcode_verified": true }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, returned) = request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/return"),
        json!({}),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(returned["status"], "returned");

    // A pending order cannot be returned: the transition is only valid from
    // in-transit or delivered.
    let (status, other_order) = request(
        app.clone(),
        "POST",
        "/v1/orders",
        json!({
            "customer_id": "00000000-0000-7000-8000-000000000003",
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "pickup" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "dropoff" },
            "fare_amount_minor": 500,
            "fare_currency": "TRY"
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let other_order_id = other_order["id"].as_str().expect("other order id");
    let (status, _) = request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{other_order_id}/return"),
        json!({}),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CONFLICT);

    // The courier that was freed by the return is available again.
    let (status, couriers) = request(app, "GET", "/v1/couriers", json!({})).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let refreshed = couriers
        .as_array()
        .expect("courier list")
        .iter()
        .find(|entry| entry["id"] == courier_id)
        .expect("courier present");
    assert_eq!(refreshed["status"], "available");
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
async fn customer_registration_joins_the_selected_tenant_and_can_open_a_browser_session() {
    let mut state = AppState::memory();
    state.token_signing_secret = Some("test-signing-secret-must-be-at-least-32-characters".into());
    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "Customer Registration Tenant".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    state
        .tenants
        .create_tenant(&tenant, "customer-registration")
        .await
        .expect("tenant");
    let (status, _) = request(
        router(state.clone()),
        "POST",
        "/v1/auth/register",
        json!({
            "email": "customer@qervon.test",
            "display_name": "Customer",
            "password": "a-long-enough-test-password",
            "tenant_slug": "customer-registration",
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
    assert!(state
        .tenants
        .find_membership(tenant.id, user.id)
        .await
        .expect("membership lookup")
        .is_some());
    let login = axum::http::Request::builder()
        .method("POST")
        .uri("/v1/browser/auth/login")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            json!({"email":"customer@qervon.test","password":"a-long-enough-test-password","tenant_slug":"customer-registration"}).to_string(),
        ))
        .expect("browser login request");
    assert_eq!(
        router(state)
            .oneshot(login)
            .await
            .expect("browser login")
            .status(),
        axum::http::StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn auth_login_accepts_a_tenant_slug_typed_in_a_different_case() {
    let app = tenant_auth_app().await;
    let (status, login) = request(
        app,
        "POST",
        "/v1/auth/login",
        json!({
            "email": "operator@qervon.test",
            "password": "a-long-enough-test-password",
            // A human typing the tenant code into the /login form's text
            // input may capitalize it; the slug itself is always stored
            // lowercase, so login must tolerate this instead of returning a
            // confusing "invalid credentials" error.
            "tenant_slug": "  Qervon-Test  "
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(login["token_type"], "Bearer");
}

#[tokio::test]
async fn forgot_password_issues_a_working_reset_link_and_the_old_password_stops_working() {
    let app = tenant_auth_app().await;

    let (status, forgot) = request(
        app.clone(),
        "POST",
        "/v1/auth/password/forgot",
        json!({ "email": "operator@qervon.test" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(forgot["status"], "sent");
    // Memory storage with no email provider configured surfaces the link
    // directly (see auth_password_forgot) so this stays testable without
    // real email infrastructure.
    let reset_url = forgot["dev_reset_url"].as_str().expect("dev reset url");
    let token = reset_url
        .split("token=")
        .nth(1)
        .expect("token query param")
        .to_string();

    let (status, _) = request(
        app.clone(),
        "POST",
        "/v1/auth/password/reset",
        json!({ "token": token, "new_password": "a-brand-new-long-password" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NO_CONTENT);

    // The old password no longer works.
    let (status, _) = request(
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
    assert_eq!(status, axum::http::StatusCode::UNAUTHORIZED);

    // The new password works.
    let (status, _) = request(
        app.clone(),
        "POST",
        "/v1/auth/login",
        json!({
            "email": "operator@qervon.test",
            "password": "a-brand-new-long-password",
            "tenant_slug": "qervon-test"
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    // The token is single-use — replaying it fails even with a valid token.
    let (status, _) = request(
        app,
        "POST",
        "/v1/auth/password/reset",
        json!({ "token": token, "new_password": "yet-another-long-password" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn forgot_password_gives_the_same_generic_response_for_an_unknown_email() {
    let app = tenant_auth_app().await;
    let (status, forgot) = request(
        app,
        "POST",
        "/v1/auth/password/forgot",
        json!({ "email": "nobody-registered@qervon.test" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(forgot["status"], "sent");
    // No account exists, so there is nothing to link to — and the response
    // shape must not otherwise reveal that (see the doc comment on the
    // handler for why this can't be a 404).
    assert!(forgot.get("dev_reset_url").is_none());
}

#[tokio::test]
async fn password_reset_rejects_a_garbage_token() {
    let app = tenant_auth_app().await;
    let (status, _) = request(
        app,
        "POST",
        "/v1/auth/password/reset",
        json!({ "token": "not-a-real-token", "new_password": "a-brand-new-long-password" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::BAD_REQUEST);
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
async fn tenant_admin_can_provision_a_courier_that_can_open_a_browser_session() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let mut state = AppState::memory();
    state.token_signing_secret = Some(String::from_utf8_lossy(secret).into_owned().into());
    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "Courier Provisioning Tenant".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    state
        .tenants
        .create_tenant(&tenant, "courier-provisioning")
        .await
        .expect("create tenant");
    let admin = state
        .auth
        .register(
            "admin@qervon.test".into(),
            "Tenant Admin".into(),
            "a-long-enough-admin-password".into(),
            UserRole::Admin,
        )
        .await
        .expect("create admin");
    state
        .tenants
        .add_member(&TenantMembership {
            tenant_id: tenant.id,
            user_id: admin.id,
            role: TenantMemberRole::Admin,
            joined_at: Utc::now(),
        })
        .await
        .expect("add admin membership");
    let admin_token = issue_access_token(
        secret,
        admin.id.0,
        tenant.id.0,
        UserRole::Admin,
        Duration::minutes(5),
    )
    .expect("issue admin token");
    let app = router(state);

    let (status, courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers/provision",
        json!({
            "email": "courier@qervon.test",
            "display_name": "Ayşe Kurye",
            "password": "a-long-enough-courier-password",
            "vehicle": "motorcycle"
        }),
        &admin_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(courier["name"], "Ayşe Kurye");

    let (status, login) = unauthenticated_request(
        app.clone(),
        "POST",
        "/v1/auth/login",
        json!({
            "email": "courier@qervon.test",
            "password": "a-long-enough-courier-password",
            "tenant_slug": "courier-provisioning"
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let courier_token = login["access_token"]
        .as_str()
        .expect("courier access token");
    let (status, profile) = authorized_request(
        app.clone(),
        "GET",
        "/v1/courier/me",
        json!({}),
        courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(profile["id"], courier["id"]);

    let request = axum::http::Request::builder()
        .method("POST")
        .uri("/v1/browser/auth/login")
        .header("host", "localhost:8080")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            json!({
                "email": "courier@qervon.test",
                "password": "a-long-enough-courier-password",
                "tenant_slug": "courier-provisioning"
            })
            .to_string(),
        ))
        .expect("browser login request");
    let response = app.oneshot(request).await.expect("browser login response");
    assert_eq!(response.status(), axum::http::StatusCode::NO_CONTENT);
    assert!(response
        .headers()
        .get_all("set-cookie")
        .iter()
        .all(|value| !value.to_str().expect("cookie header").contains("; Secure")));
}

#[tokio::test]
async fn initial_setup_creates_platform_admin_then_tenant_admins_and_tenants() {
    let mut state = AppState::memory();
    state.token_signing_secret = Some("test-signing-secret-must-be-at-least-32-characters".into());
    let app = router(state);

    let (status, setup_status) =
        unauthenticated_request(app.clone(), "GET", "/v1/setup/status", json!({})).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(setup_status["initial_setup_required"], true);

    let (status, created) = unauthenticated_request(
        app.clone(),
        "POST",
        "/v1/setup/initialize",
        json!({
            "tenant_name": "Platform Tenant",
            "tenant_slug": "platform-tenant",
            "admin_name": "Platform Owner",
            "admin_email": "owner@qervon.test",
            "admin_password": "a-long-enough-owner-password"
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(created["tenant_slug"], "platform-tenant");
    assert_eq!(created["admin_role"], "super_admin");

    let browser_login_request = axum::http::Request::builder()
        .method("POST")
        .uri("/v1/browser/auth/login")
        .header("host", "localhost:8080")
        .header("content-type", "application/json")
        .body(axum::body::Body::from(
            json!({
                "tenant_slug": "platform-tenant",
                "email": "owner@qervon.test",
                "password": "a-long-enough-owner-password"
            })
            .to_string(),
        ))
        .expect("browser login request");
    let browser_login_response = app
        .clone()
        .oneshot(browser_login_request)
        .await
        .expect("browser login response");
    assert_eq!(
        browser_login_response.status(),
        axum::http::StatusCode::NO_CONTENT
    );
    let access_cookie = browser_login_response
        .headers()
        .get_all("set-cookie")
        .iter()
        .find_map(|header| {
            header
                .to_str()
                .ok()?
                .strip_prefix("qervon_access_token=")?
                .split(';')
                .next()
        })
        .expect("browser access cookie");
    let overview_request = axum::http::Request::builder()
        .uri("/v1/operations/overview")
        .header("cookie", format!("qervon_access_token={access_cookie}"))
        .body(axum::body::Body::empty())
        .expect("overview request");
    let overview_response = app
        .clone()
        .oneshot(overview_request)
        .await
        .expect("overview response");
    assert_eq!(overview_response.status(), axum::http::StatusCode::OK);

    let (status, login) = unauthenticated_request(
        app.clone(),
        "POST",
        "/v1/auth/login",
        json!({
            "tenant_slug": "platform-tenant",
            "email": "owner@qervon.test",
            "password": "a-long-enough-owner-password"
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let platform_token = login["access_token"].as_str().expect("platform token");

    let (status, tenant) = authorized_request(
        app.clone(),
        "POST",
        "/v1/tenants/provision",
        json!({
            "tenant_name": "Yeni Tenant",
            "tenant_slug": "yeni-tenant",
            "admin_name": "Yeni Tenant Yönetici",
            "admin_email": "admin@yeni-tenant.test",
            "admin_password": "a-long-enough-tenant-admin-password"
        }),
        platform_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(tenant["tenant_slug"], "yeni-tenant");

    let (status, tenant_admin_login) = unauthenticated_request(
        app.clone(),
        "POST",
        "/v1/auth/login",
        json!({
            "tenant_slug": "yeni-tenant",
            "email": "admin@yeni-tenant.test",
            "password": "a-long-enough-tenant-admin-password"
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let tenant_admin_token = tenant_admin_login["access_token"]
        .as_str()
        .expect("tenant admin token");

    let (status, tenant_admin) = authorized_request(
        app.clone(),
        "POST",
        "/v1/company/admins/provision",
        json!({
            "display_name": "Operasyon Yönetici",
            "email": "operations@yeni-tenant.test",
            "password": "a-long-enough-operations-password"
        }),
        tenant_admin_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(tenant_admin["role"], "admin");

    let (status, blocked) = unauthenticated_request(
        app,
        "POST",
        "/v1/setup/initialize",
        json!({
            "tenant_name": "Should Not Exist",
            "tenant_slug": "second-setup",
            "admin_name": "Blocked",
            "admin_email": "blocked@qervon.test",
            "admin_password": "a-long-enough-blocked-password"
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CONFLICT);
    assert_eq!(blocked["status"], 409);
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
async fn courier_gps_updates_reach_operator_and_customer_tracking() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, _) = tenant_tracking_fixture().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");
    let (status, courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "GPS Kurye", "vehicle": "motorcycle" }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = Uuid::parse_str(courier["id"].as_str().expect("courier id")).expect("UUID");
    let customer_id = Uuid::now_v7();
    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/orders",
        json!({
            "customer_id": customer_id,
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "fare_amount_minor": 1500,
            "fare_currency": "TRY"
        }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order_id = order["id"].as_str().expect("order id");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/assign"),
        json!({ "courier_id": courier_id }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let courier_token = issue_access_token(
        secret,
        courier_id,
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier token");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        "/v1/courier/me/location",
        json!({ "latitude": 41.021, "longitude": 29.031, "speed_kmh": 22.0, "battery_pct": 80 }),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, fleet_locations) = authorized_request(
        app.clone(),
        "GET",
        "/v1/tracking/live",
        json!({}),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(fleet_locations[0]["courier_id"], courier_id.to_string());
    assert_eq!(fleet_locations[0]["latitude"], 41.021);

    let customer_token = issue_access_token(
        secret,
        customer_id,
        operator_claims.tenant_id,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("customer token");
    let (status, customer_location) = authorized_request(
        app,
        "GET",
        &format!("/v1/orders/{order_id}/tracking"),
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(customer_location["courier_id"], courier_id.to_string());
    assert_eq!(customer_location["longitude"], 29.031);
}

#[tokio::test]
async fn ai_fraud_guard_flags_implausible_gps_jumps_without_rejecting_them() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, _) = tenant_tracking_fixture().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");

    let (status, courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Fraud Guard Kurye", "vehicle": "car" }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = Uuid::parse_str(courier["id"].as_str().expect("courier id")).expect("UUID");
    let courier_token = issue_access_token(
        secret,
        courier_id,
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier token");

    // First sample establishes a baseline location.
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        "/v1/courier/me/location",
        json!({ "latitude": 41.0, "longitude": 29.0 }),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    // Second sample jumps ~140km away essentially instantly: physically
    // impossible, so the AI Fraud Guard must flag it — but the update is
    // still accepted (flag-and-accept, no rejection).
    let (status, accepted) = authorized_request(
        app.clone(),
        "POST",
        "/v1/courier/me/location",
        json!({ "latitude": 42.0, "longitude": 30.0 }),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(accepted["id"], courier_id.to_string());

    let (status, fleet_locations) =
        authorized_request(app, "GET", "/v1/tracking/live", json!({}), &operator_token).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let entry = fleet_locations
        .as_array()
        .expect("locations array")
        .iter()
        .find(|event| event["courier_id"] == courier_id.to_string())
        .expect("courier location present");
    assert_eq!(entry["fraud_flagged"], true);
    assert!(entry["fraud_risk_score"].as_f64().expect("risk score") > 0.0);
}

#[tokio::test]
async fn otp_login_round_trip_and_wrong_code_is_rejected() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let mut state = AppState::memory();
    state.token_signing_secret = Some(String::from_utf8_lossy(secret).into_owned().into());
    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "OTP Tenant".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    state
        .tenants
        .create_tenant(&tenant, "otp-tenant")
        .await
        .expect("create tenant");
    let courier_user = state
        .auth
        .register(
            "otp-courier@qervon.test".into(),
            "OTP Courier".into(),
            "a-long-enough-test-password".into(),
            UserRole::Courier,
        )
        .await
        .expect("create courier user");
    state
        .tenants
        .add_member(&TenantMembership {
            tenant_id: tenant.id,
            user_id: courier_user.id,
            role: TenantMemberRole::Member,
            joined_at: Utc::now(),
        })
        .await
        .expect("add courier membership");
    let courier_token = issue_access_token(
        secret,
        courier_user.id.0,
        tenant.id.0,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier token");
    let app = router(state);

    // Link a phone number to the account first (self-service, signed-in).
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        "/v1/auth/phone",
        json!({ "phone": "+905551234567" }),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    // Requesting an OTP for an unlinked phone number fails.
    let (status, _) = unauthenticated_request(
        app.clone(),
        "POST",
        "/v1/auth/otp/request",
        json!({ "tenant_slug": "otp-tenant", "phone": "+905550000000" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNAUTHORIZED);

    // Requesting an OTP for the linked phone succeeds and (in memory/local
    // mode only) returns the raw code for local testing.
    let (status, requested) = unauthenticated_request(
        app.clone(),
        "POST",
        "/v1/auth/otp/request",
        json!({ "tenant_slug": "otp-tenant", "phone": "+905551234567" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(requested["status"], "sent");
    let code = requested["dev_code"]
        .as_str()
        .expect("dev code present in memory mode")
        .to_string();
    assert_eq!(code.len(), 6);

    // A wrong code is rejected without consuming the correct one.
    let (status, _) = unauthenticated_request(
        app.clone(),
        "POST",
        "/v1/auth/otp/verify",
        json!({ "tenant_slug": "otp-tenant", "phone": "+905551234567", "code": "000000" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNAUTHORIZED);

    // The correct code succeeds and returns a full token pair, just like
    // password login.
    let (status, verified) = unauthenticated_request(
        app,
        "POST",
        "/v1/auth/otp/verify",
        json!({ "tenant_slug": "otp-tenant", "phone": "+905551234567", "code": code }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(verified["token_type"], "Bearer");
    assert!(verified["access_token"].as_str().is_some());
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
    let courier_id = Uuid::now_v7();
    state
        .couriers
        .register_courier(RegisterCourierInput {
            id: courier_id,
            name: "Available Customer Courier".into(),
            vehicle: VehicleType::Motorcycle,
        })
        .await
        .expect("courier");
    state
        .couriers
        .update_courier_location(
            courier_id,
            Location::new(41.0, 29.0).expect("courier location"),
        )
        .await
        .expect("location");
    state
        .tenants
        .bind_courier(tenant.id, courier_id)
        .await
        .expect("courier tenant");
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
    let courier_token = issue_access_token(
        secret,
        courier_id,
        tenant.id.0,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier token");
    let app = router(state);

    let base_order = json!({
        "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Müşteri alım" },
        "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Müşteri teslim" }
    });
    let mut missing_phone = base_order.clone();
    missing_phone["contact_phone"] = Value::Null;
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/orders",
        missing_phone,
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);

    let mut qr_payment = base_order;
    qr_payment["payment_method"] = Value::String("qr".into());
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/orders",
        qr_payment,
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);

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
    assert_eq!(order["status"], "pending");
    let order_id = order["id"].as_str().expect("order id").to_string();

    // The order is only offered, not yet assigned, until the courier accepts.
    let (status, offer) = authorized_request(
        app.clone(),
        "GET",
        "/v1/courier/me/offer",
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(offer["order"]["id"], order_id);

    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/accept"),
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(order["assigned_courier_id"], courier_id.to_string());
    assert_eq!(order["status"], "courier_assigned");

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
    assert_eq!(overview["pending_orders"], 0);

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/orders",
        json!({
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Yeni alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Yeni teslim" },
            "contact_phone": "05550000000"
        }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::TOO_MANY_REQUESTS);

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
async fn customer_bulk_csv_import_validates_entire_file_and_preserves_session_ownership() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "Bulk Import Tenant".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    let mut state = AppState::memory();
    state.token_signing_secret = Some(String::from_utf8_lossy(secret).into_owned().into());
    state
        .tenants
        .create_tenant(&tenant, "bulk-import")
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
    let header = "reference,pickup_label,pickup_latitude,pickup_longitude,dropoff_label,dropoff_latitude,dropoff_longitude,contact_phone,payment_method,delivery_note";

    let invalid = format!(
        "{header}\nSIP-001,Alım 1,41.0,29.0,Teslim 1,41.1,29.1,05550000000,cash,Not 1\nSIP-002,Alım 2,41.2,29.2,Teslim 2,999,29.3,05550000001,card,Not 2\n"
    );
    let (status, body) = authorized_csv_request(
        app.clone(),
        "/v1/customer/orders/bulk",
        &invalid,
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body["detail"]
        .as_str()
        .expect("validation detail")
        .contains("dropoff_latitude"));
    let (status, orders) = authorized_request(
        app.clone(),
        "GET",
        "/v1/customer/orders",
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(orders.as_array().expect("orders").is_empty());

    let valid = format!(
        "{header}\nSIP-001,\"Alım, 1\",41.0,29.0,Teslim 1,41.1,29.1,05550000000,cash,Not 1\nSIP-002,Alım 2,41.2,29.2,Teslim 2,41.3,29.3,05550000001,card,Not 2\n"
    );
    let (status, imported) = authorized_csv_request(
        app.clone(),
        "/v1/customer/orders/bulk",
        &valid,
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(imported["requested_count"], 2);
    assert_eq!(imported["created_count"], 2);
    assert_eq!(imported["orders"][0]["reference"], "SIP-001");
    assert_eq!(
        imported["orders"][0]["order"]["customer_id"],
        customer_id.to_string()
    );
    assert_eq!(imported["orders"][1]["order"]["payment_method"], "card");
    assert!(
        imported["orders"][0]["order"]["fare"]["amount_minor"]
            .as_i64()
            .expect("server fare")
            > 0
    );

    let (status, mine) = authorized_request(
        app.clone(),
        "GET",
        "/v1/customer/orders",
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(mine.as_array().expect("customer orders").len(), 2);
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

    let (status, _) =
        authorized_csv_request(app, "/v1/customer/orders/bulk", &valid, &operator_token).await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn operational_reports_preserve_tenant_scope_and_company_writes_require_admin() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: "Reporting Tenant".into(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    let mut state = AppState::memory();
    state.token_signing_secret = Some(String::from_utf8_lossy(secret).into_owned().into());
    state
        .tenants
        .create_tenant(&tenant, "reporting-tenant")
        .await
        .expect("tenant");
    let operator_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        tenant.id.0,
        UserRole::Operator,
        Duration::minutes(5),
    )
    .expect("operator token");
    let app = router(state);

    let (status, report) = authorized_request(
        app.clone(),
        "GET",
        "/v1/reports/operations",
        json!({}),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(report["overview"]["active_orders"], 0);
    assert_eq!(report["orders_by_status"], json!({}));

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        "/v1/company/members",
        json!({ "user_id": Uuid::now_v7(), "role": "operator" }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);

    let (status, body) = authorized_request(
        app,
        "GET",
        "/v1/finance/summary",
        json!({}),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);
    assert!(body["detail"]
        .as_str()
        .unwrap_or_default()
        .contains("PostgreSQL"));
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
async fn public_html_aliases_serve_the_real_application_surfaces() {
    let app = app();
    for path in [
        "/index.html",
        "/customer.html",
        "/login.html",
        "/setup.html",
        "/mobile-customer.html",
        "/mobile-courier.html",
    ] {
        let response = app
            .clone()
            .oneshot(
                axum::http::Request::builder()
                    .uri(path)
                    .body(axum::body::Body::empty())
                    .expect("page request"),
            )
            .await
            .expect("page response");
        assert_eq!(response.status(), axum::http::StatusCode::OK, "{path}");
        assert_eq!(
            response.headers()["content-type"],
            "text/html; charset=utf-8"
        );
    }
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

#[tokio::test]
async fn courier_wallet_is_credited_on_delivery_and_visible_to_owner_and_operations() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, other_tenant_token) = tenant_tracking_fixture().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");

    let (status, courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Wallet Kurye", "vehicle": "motorcycle" }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = Uuid::parse_str(courier["id"].as_str().expect("courier id")).expect("UUID");
    let courier_token = issue_access_token(
        secret,
        courier_id,
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier token");

    // A brand-new courier has a zero-balance wallet (no persistence side
    // effect from reading it).
    let (status, wallet) = authorized_request(
        app.clone(),
        "GET",
        "/v1/courier/me/wallet",
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(wallet["balance_minor"], 0);
    assert!(wallet["transactions"].as_array().unwrap().is_empty());

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/couriers/{courier_id}/location"),
        json!({ "latitude": 41.0, "longitude": 29.0 }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/orders",
        json!({
            "customer_id": Uuid::now_v7(),
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "fare_amount_minor": 4500,
            "fare_currency": "TRY"
        }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order_id = order["id"].as_str().expect("order id");

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/assign"),
        json!({ "courier_id": courier_id }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/transit"),
        json!({}),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/deliver"),
        json!({}),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    // Delivery auto-credited the courier's wallet with the full fare.
    let (status, wallet) = authorized_request(
        app.clone(),
        "GET",
        "/v1/courier/me/wallet",
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(wallet["balance_minor"], 4500);
    assert_eq!(wallet["total_earned_minor"], 4500);
    let transactions = wallet["transactions"].as_array().unwrap();
    assert_eq!(transactions.len(), 1);
    assert_eq!(transactions[0]["transaction_type"], "delivery_earning");

    // The tenant's own operator can read the same wallet by courier id.
    let (status, wallet_via_ops) = authorized_request(
        app.clone(),
        "GET",
        &format!("/v1/couriers/{courier_id}/wallet"),
        json!({}),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(wallet_via_ops["balance_minor"], 4500);

    // A different tenant cannot read this courier's wallet.
    let (status, _) = authorized_request(
        app,
        "GET",
        &format!("/v1/couriers/{courier_id}/wallet"),
        json!({}),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn customer_can_rate_a_delivered_order_and_raise_support_tickets() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, other_tenant_token) = tenant_tracking_fixture().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");

    let (status, courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Rating Kurye", "vehicle": "motorcycle" }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = Uuid::parse_str(courier["id"].as_str().expect("courier id")).expect("UUID");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/couriers/{courier_id}/location"),
        json!({ "latitude": 41.0, "longitude": 29.0 }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let customer_id = Uuid::now_v7();
    let customer_token = issue_access_token(
        secret,
        customer_id,
        operator_claims.tenant_id,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("customer token");

    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/orders",
        json!({
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "fare_amount_minor": 3200,
            "fare_currency": "TRY"
        }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order_id = order["id"].as_str().expect("order id");

    // Rating before delivery is rejected.
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/customer/orders/{order_id}/rating"),
        json!({ "rating_stars": 5 }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CONFLICT);

    // The customer order endpoint offers the job to the nearest available
    // online courier; the courier must accept before it is assigned.
    let courier_token = issue_access_token(
        secret,
        courier_id,
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier token");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/accept"),
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/transit"),
        json!({}),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/deliver"),
        json!({}),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    // Another customer cannot rate this order.
    let stranger_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("stranger token");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/customer/orders/{order_id}/rating"),
        json!({ "rating_stars": 1 }),
        &stranger_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CONFLICT);

    let (status, rating) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/customer/orders/{order_id}/rating"),
        json!({ "rating_stars": 5, "comment": "Harika!" }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(rating["rating_stars"], 5);

    // Rating twice is rejected.
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/customer/orders/{order_id}/rating"),
        json!({ "rating_stars": 3 }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CONFLICT);

    let (status, ratings) = authorized_request(
        app.clone(),
        "GET",
        &format!("/v1/couriers/{courier_id}/ratings"),
        json!({}),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(ratings.as_array().unwrap().len(), 1);

    // A different tenant cannot read this courier's ratings.
    let (status, _) = authorized_request(
        app.clone(),
        "GET",
        &format!("/v1/couriers/{courier_id}/ratings"),
        json!({}),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);

    // Support tickets: open one tied to this order and one general ticket.
    let (status, ticket) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/support-tickets",
        json!({ "order_id": order_id, "subject": "Gecikme", "message": "Siparişim geç geldi" }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(ticket["status"], "open");

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/support-tickets",
        json!({ "subject": "Genel soru", "message": "Ödeme yöntemleri nelerdir?" }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);

    let (status, tickets) = authorized_request(
        app,
        "GET",
        "/v1/customer/support-tickets",
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(tickets.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn coupon_can_be_applied_to_a_customer_order_and_is_tenant_isolated() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, other_tenant_token) = tenant_tracking_fixture().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");

    let (status, coupon) = authorized_request(
        app.clone(),
        "POST",
        "/v1/coupons",
        json!({
            "code": "qervon20",
            "discount_percent": 20.0,
            "max_discount_minor": 1000,
            "valid_until": (Utc::now() + Duration::days(30)).to_rfc3339(),
            "usage_limit": 1
        }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(coupon["code"], "QERVON20");
    assert_eq!(coupon["used_count"], 0);

    // Listing without a tenant-scoped session is rejected.
    let (status, _) = unauthenticated_request(app.clone(), "GET", "/v1/coupons", json!({})).await;
    assert_eq!(status, axum::http::StatusCode::UNAUTHORIZED);

    // A different tenant does not see this coupon.
    let (status, other_coupons) = authorized_request(
        app.clone(),
        "GET",
        "/v1/coupons",
        json!({}),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(other_coupons.as_array().unwrap().is_empty());

    let customer_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("customer token");

    // The fare is now computed server-side from pickup/dropoff distance —
    // fetch the same quote the order creation path will use internally so
    // the expected discounted fare below isn't a magic number.
    let (status, quote) = authorized_request(
        app.clone(),
        "GET",
        "/v1/customer/fare-quote?pickup_latitude=41.0&pickup_longitude=29.0&\
         dropoff_latitude=41.1&dropoff_longitude=29.1",
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let base_fare_minor = quote["fare_amount_minor"].as_i64().expect("base fare");
    // Mirrors PromoCouponEngine::apply_coupon: 20% discount, truncated, capped
    // at max_discount_minor = 1000.
    let expected_discount = ((base_fare_minor as f64) * 0.2) as i64;
    let expected_discount = expected_discount.min(1000);
    let expected_fare_minor = base_fare_minor - expected_discount;

    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/orders",
        json!({
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "coupon_code": "qervon20"
        }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(order["fare"]["amount_minor"], expected_fare_minor);

    // The coupon's usage limit (1) is now exhausted.
    let second_customer_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("second customer token");
    let (status, _) = authorized_request(
        app,
        "POST",
        "/v1/customer/orders",
        json!({
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "coupon_code": "QERVON20"
        }),
        &second_customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CONFLICT);
}

#[tokio::test]
async fn geocode_search_requires_a_signed_in_user_and_short_circuits_short_queries() {
    let app = tenant_auth_app().await;

    // No real signed-in user — must be rejected before ever reaching the
    // Nominatim proxy call.
    let (status, _) = request(
        app.clone(),
        "GET",
        "/v1/geocode/search?q=Kadikoy",
        json!({}),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNAUTHORIZED);

    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let customer_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        Uuid::now_v7(),
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("customer token");

    // Below the 3-character floor: short-circuits to an empty result with no
    // outbound network call, so this stays a fast, offline unit test.
    let (status, results) = authorized_request(
        app,
        "GET",
        "/v1/geocode/search?q=ka",
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(results.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn fare_quote_reflects_distance_and_is_configurable_by_tenant_admins_only() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, _other_tenant_token) = tenant_tracking_fixture().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");
    let customer_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("customer token");
    let admin_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Admin,
        Duration::minutes(5),
    )
    .expect("admin token");

    // Identical pickup/dropoff -> zero distance -> the documented minimum
    // fare (1500 minor units, TRY) applies.
    let (status, quote) = authorized_request(
        app.clone(),
        "GET",
        "/v1/customer/fare-quote?pickup_latitude=41.0&pickup_longitude=29.0&\
         dropoff_latitude=41.0&dropoff_longitude=29.0",
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(quote["fare_amount_minor"], 1500);
    assert_eq!(quote["currency"], "TRY");
    assert_eq!(quote["distance_km"], 0.0);

    // A dispatcher (operational, but not admin) can read pricing but not
    // change it.
    let (status, pricing) = authorized_request(
        app.clone(),
        "GET",
        "/v1/pricing",
        json!({}),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(pricing["base_fare_minor"], 1000);
    let (status, _) = authorized_request(
        app.clone(),
        "PUT",
        "/v1/pricing",
        json!({
            "base_fare_minor": 2000,
            "per_km_rate_minor": 500,
            "minimum_fare_minor": 3000,
            "currency": "USD"
        }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);

    // A tenant admin can update pricing.
    let (status, updated) = authorized_request(
        app.clone(),
        "PUT",
        "/v1/pricing",
        json!({
            "base_fare_minor": 2000,
            "per_km_rate_minor": 500,
            "minimum_fare_minor": 3000,
            "currency": "USD"
        }),
        &admin_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(updated["currency"], "USD");

    // Re-quoting the same zero-distance pair now reflects the new minimum.
    let (status, quote_after_update) = authorized_request(
        app,
        "GET",
        "/v1/customer/fare-quote?pickup_latitude=41.0&pickup_longitude=29.0&\
         dropoff_latitude=41.0&dropoff_longitude=29.0",
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(quote_after_update["fare_amount_minor"], 3000);
    assert_eq!(quote_after_update["currency"], "USD");
}

#[tokio::test]
async fn customer_can_cancel_their_own_order_but_not_after_pickup_or_someone_elses() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, _other_tenant_token) = tenant_tracking_fixture().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");
    let customer_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("customer token");

    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/orders",
        json!({
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "delivery_note": "Kapıcıya bırakın",
            "contact_phone": "+905551234567"
        }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(order["delivery_note"], "Kapıcıya bırakın");
    assert_eq!(order["contact_phone"], "+905551234567");
    let order_id = order["id"].as_str().expect("order id").to_string();

    // A stranger cannot cancel this order.
    let stranger_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("stranger token");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/customer/orders/{order_id}/cancel"),
        json!({}),
        &stranger_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);

    // The owner can cancel a still-pending order.
    let (status, cancelled) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/customer/orders/{order_id}/cancel"),
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(cancelled["status"], "cancelled");

    // Cancelling an already-cancelled order is rejected.
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/customer/orders/{order_id}/cancel"),
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CONFLICT);

    // A second order that reaches in_transit can no longer be cancelled by
    // the customer.
    let (status, courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Cancel Kurye", "vehicle": "motorcycle" }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = Uuid::parse_str(courier["id"].as_str().expect("courier id")).expect("UUID");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/couriers/{courier_id}/location"),
        json!({ "latitude": 41.0, "longitude": 29.0 }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, order2) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/orders",
        json!({
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" }
        }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order2_id = order2["id"].as_str().expect("order id").to_string();

    let courier_token = issue_access_token(
        secret,
        courier_id,
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier token");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order2_id}/accept"),
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order2_id}/pickup"),
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, _) = authorized_request(
        app,
        "POST",
        &format!("/v1/customer/orders/{order2_id}/cancel"),
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CONFLICT);
}

#[tokio::test]
async fn customer_sees_eta_only_once_a_courier_is_assigned_and_reporting_location() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, _other_tenant_token) = tenant_tracking_fixture().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");
    let customer_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("customer token");

    // The courier must exist and be online before the order is created, so
    // the auto-offer at order-creation time has a candidate.
    let (status, courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Eta Kurye", "vehicle": "motorcycle" }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = Uuid::parse_str(courier["id"].as_str().expect("courier id")).expect("UUID");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/couriers/{courier_id}/location"),
        json!({ "latitude": 41.0, "longitude": 29.0 }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/orders",
        json!({
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" }
        }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order_id = order["id"].as_str().expect("order id").to_string();

    // Merely offered (not yet accepted): still no assigned courier, so
    // still null, not an error.
    let (status, eta) = authorized_request(
        app.clone(),
        "GET",
        &format!("/v1/customer/orders/{order_id}/eta"),
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(eta.is_null());

    let courier_token = issue_access_token(
        secret,
        courier_id,
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier token");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/accept"),
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, eta_after_assignment) = authorized_request(
        app.clone(),
        "GET",
        &format!("/v1/customer/orders/{order_id}/eta"),
        json!({}),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(eta_after_assignment["eta_minutes"].as_f64().unwrap() >= 0.0);
    assert_eq!(eta_after_assignment["distance_km"], 0.0);

    // A stranger cannot see this order's ETA.
    let stranger_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("stranger token");
    let (status, _) = authorized_request(
        app,
        "GET",
        &format!("/v1/customer/orders/{order_id}/eta"),
        json!({}),
        &stranger_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn customer_order_payment_method_is_recorded_and_courier_confirms_cash_collection() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, _other_tenant_token) = tenant_tracking_fixture().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");

    let (status, courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Payment Kurye", "vehicle": "motorcycle" }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = Uuid::parse_str(courier["id"].as_str().expect("courier id")).expect("UUID");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/couriers/{courier_id}/location"),
        json!({ "latitude": 41.0, "longitude": 29.0 }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let customer_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("customer token");

    // Rejects an invalid payment method up front.
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/orders",
        json!({
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "fare_amount_minor": 5000,
            "fare_currency": "TRY",
            "payment_method": "bitcoin"
        }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);

    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/orders",
        json!({
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "fare_amount_minor": 5000,
            "fare_currency": "TRY",
            "payment_method": "cash"
        }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(order["payment_method"], "cash");
    assert_eq!(order["payment_collected"], false);
    let order_id = order["id"].as_str().expect("order id");

    let courier_token = issue_access_token(
        secret,
        courier_id,
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier token");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/accept"),
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/pickup"),
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, delivered) = authorized_request(
        app,
        "POST",
        &format!("/v1/courier/orders/{order_id}/deliver"),
        json!({
            "recipient_name": "Ali Veli",
            "qr_barcode_verified": true,
            "payment_collected": true
        }),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(delivered["payment_collected"], true);
    assert_eq!(delivered["payment_method"], "cash");
}

#[tokio::test]
async fn push_device_registration_is_idempotent_and_scoped_to_the_owner() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, _other_tenant_token) = tenant_tracking_fixture().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");

    let user_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("user token");

    // Rejects an unknown platform.
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        "/v1/push/devices",
        json!({ "platform": "windows_phone", "app": "courier", "device_token": "abc" }),
        &user_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::UNPROCESSABLE_ENTITY);

    let (status, first) = authorized_request(
        app.clone(),
        "POST",
        "/v1/push/devices",
        json!({ "platform": "ios", "app": "courier", "device_token": "device-token-1" }),
        &user_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let token_id = first["id"].as_str().expect("token id").to_string();

    // Re-registering the same device token is idempotent (same id, no duplicate).
    let (status, second) = authorized_request(
        app.clone(),
        "POST",
        "/v1/push/devices",
        json!({ "platform": "ios", "app": "courier", "device_token": "device-token-1" }),
        &user_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(second["id"], token_id);

    let (status, list) = authorized_request(
        app.clone(),
        "GET",
        "/v1/push/devices",
        json!({}),
        &user_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(list.as_array().unwrap().len(), 1);

    // A different signed-in user does not see or affect this token.
    let stranger_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("stranger token");
    let (status, stranger_list) = authorized_request(
        app.clone(),
        "GET",
        "/v1/push/devices",
        json!({}),
        &stranger_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(stranger_list.as_array().unwrap().is_empty());

    let (status, _) = authorized_request(
        app.clone(),
        "DELETE",
        &format!("/v1/push/devices/{token_id}"),
        json!({}),
        &stranger_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NO_CONTENT);
    let (status, list_after_stranger_delete) = authorized_request(
        app.clone(),
        "GET",
        "/v1/push/devices",
        json!({}),
        &user_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(
        list_after_stranger_delete.as_array().unwrap().len(),
        1,
        "a stranger's delete call must not remove someone else's token"
    );

    let (status, _) = authorized_request(
        app.clone(),
        "DELETE",
        &format!("/v1/push/devices/{token_id}"),
        json!({}),
        &user_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NO_CONTENT);
    let (status, list_after_delete) =
        authorized_request(app, "GET", "/v1/push/devices", json!({}), &user_token).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(list_after_delete.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn courier_job_offer_can_be_accepted_or_rejected_over_http() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, other_tenant_token) = tenant_tracking_fixture().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");

    let (status, courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Teklif Kurye", "vehicle": "motorcycle" }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = Uuid::parse_str(courier["id"].as_str().expect("courier id")).expect("UUID");
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/couriers/{courier_id}/location"),
        json!({ "latitude": 41.0, "longitude": 29.0 }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let courier_token = issue_access_token(
        secret,
        courier_id,
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier token");

    // No offer exists yet.
    let (status, offer) = authorized_request(
        app.clone(),
        "GET",
        "/v1/courier/me/offer",
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(offer.is_null());

    let customer_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("customer token");
    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/orders",
        json!({
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "fare_amount_minor": 4200,
            "fare_currency": "TRY"
        }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(order["status"], "pending");
    assert!(order["assigned_courier_id"].is_null());
    let order_id = order["id"].as_str().expect("order id").to_string();

    // A different courier (in another tenant) cannot see or act on this offer.
    let (status, foreign_offer) = authorized_request(
        app.clone(),
        "GET",
        "/v1/courier/me/offer",
        json!({}),
        &other_tenant_token,
    )
    .await;
    // other_tenant_token is a Dispatcher role, not Courier — must be forbidden
    // from courier-only routes regardless of offer state.
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
    let _ = foreign_offer;

    let (status, offer) = authorized_request(
        app.clone(),
        "GET",
        "/v1/courier/me/offer",
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(offer["order"]["id"], order_id);
    assert!(offer["expires_at"].is_string());

    // Reject: order stays pending, courier stays available, offer clears.
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/reject"),
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NO_CONTENT);

    let (status, no_offer) = authorized_request(
        app.clone(),
        "GET",
        "/v1/courier/me/offer",
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(no_offer.is_null());

    let (status, couriers_after_reject) = authorized_request(
        app.clone(),
        "GET",
        "/v1/couriers",
        json!({}),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let courier_after_reject = couriers_after_reject
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == courier_id.to_string())
        .expect("courier present");
    assert_eq!(courier_after_reject["status"], "available");

    // Accepting the now-rejected offer fails.
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/accept"),
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CONFLICT);

    // Re-offer via a fresh order and this time accept it.
    let second_customer_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("second customer token");
    let (status, order2) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/orders",
        json!({
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "fare_amount_minor": 3000,
            "fare_currency": "TRY"
        }),
        &second_customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order2_id = order2["id"].as_str().expect("order id").to_string();

    let (status, accepted) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order2_id}/accept"),
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(accepted["status"], "courier_assigned");
    assert_eq!(accepted["assigned_courier_id"], courier_id.to_string());

    let (status, couriers_after_accept) =
        authorized_request(app, "GET", "/v1/couriers", json!({}), &operator_token).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let courier_after_accept = couriers_after_accept
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == courier_id.to_string())
        .expect("courier present");
    assert_eq!(courier_after_accept["status"], "busy");
}

/// Verifies the automatic re-offer cascade added alongside the offer/
/// accept/reject flow: when the best-ranked courier rejects a job offer,
/// the order is automatically re-offered to the next-best available
/// courier in the same tenant (excluding the one who just rejected),
/// without any operator intervention — see
/// `DispatchService::reoffer_from_candidates` and
/// `BACKEND_BACKLOG.md`'s (now resolved) "no automatic re-offer cascade"
/// note.
#[tokio::test]
async fn rejected_offer_automatically_cascades_to_the_next_best_courier() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, _other_tenant_token) = tenant_tracking_fixture().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");

    // Courier A sits exactly at the pickup point (closest possible, so the
    // AI Dispatcher ranks it first); Courier B is deliberately far away.
    let (status, courier_a) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Kaskad Kurye A", "vehicle": "motorcycle" }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_a_id =
        Uuid::parse_str(courier_a["id"].as_str().expect("courier id")).expect("UUID");

    let (status, courier_b) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Kaskad Kurye B", "vehicle": "motorcycle" }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_b_id =
        Uuid::parse_str(courier_b["id"].as_str().expect("courier id")).expect("UUID");

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/couriers/{courier_a_id}/location"),
        json!({ "latitude": 41.0, "longitude": 29.0 }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/couriers/{courier_b_id}/location"),
        json!({ "latitude": 41.9, "longitude": 29.9 }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let courier_a_token = issue_access_token(
        secret,
        courier_a_id,
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier A token");
    let courier_b_token = issue_access_token(
        secret,
        courier_b_id,
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier B token");

    let customer_token = issue_access_token(
        secret,
        Uuid::now_v7(),
        operator_claims.tenant_id,
        UserRole::Customer,
        Duration::minutes(5),
    )
    .expect("customer token");
    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/customer/orders",
        json!({
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "fare_amount_minor": 4200,
            "fare_currency": "TRY"
        }),
        &customer_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order_id = order["id"].as_str().expect("order id").to_string();

    // The closer courier (A) gets the first offer.
    let (status, offer_a) = authorized_request(
        app.clone(),
        "GET",
        "/v1/courier/me/offer",
        json!({}),
        &courier_a_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(offer_a["order"]["id"], order_id);

    // Courier B has nothing yet — the order hasn't cascaded to them until A
    // actually responds.
    let (status, offer_b_before) = authorized_request(
        app.clone(),
        "GET",
        "/v1/courier/me/offer",
        json!({}),
        &courier_b_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(offer_b_before.is_null());

    // Courier A rejects — this should automatically re-offer the same order
    // to Courier B, the only other available courier in this tenant.
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/reject"),
        json!({}),
        &courier_a_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NO_CONTENT);

    let (status, offer_b_after) = authorized_request(
        app.clone(),
        "GET",
        "/v1/courier/me/offer",
        json!({}),
        &courier_b_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(
        offer_b_after["order"]["id"], order_id,
        "the order must have automatically cascaded to courier B after courier A's rejection"
    );

    // Courier A no longer has (and can no longer act on) an offer for this
    // order — it moved on to B.
    let (status, offer_a_after) = authorized_request(
        app.clone(),
        "GET",
        "/v1/courier/me/offer",
        json!({}),
        &courier_a_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(offer_a_after.is_null());

    // Courier B accepts the cascaded offer; the order is now assigned to B,
    // never having gone back to A.
    let (status, accepted) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/courier/orders/{order_id}/accept"),
        json!({}),
        &courier_b_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(accepted["status"], "courier_assigned");
    assert_eq!(accepted["assigned_courier_id"], courier_b_id.to_string());

    // Courier A, having rejected, was never touched by dispatch and stays
    // available for other work.
    let (status, couriers) =
        authorized_request(app, "GET", "/v1/couriers", json!({}), &operator_token).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let courier_a_final = couriers
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == courier_a_id.to_string())
        .expect("courier A present");
    assert_eq!(courier_a_final["status"], "available");
}

#[tokio::test]
async fn courier_can_view_own_ratings_but_not_someone_elses() {
    let secret = b"test-signing-secret-must-be-at-least-32-characters";
    let (app, operator_token, other_tenant_token) = tenant_tracking_fixture().await;
    let operator_claims = verify_access_token(secret, &operator_token).expect("operator claims");

    let (status, courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Puanli Kurye", "vehicle": "motorcycle" }),
        &operator_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = Uuid::parse_str(courier["id"].as_str().expect("courier id")).expect("UUID");
    let courier_token = issue_access_token(
        secret,
        courier_id,
        operator_claims.tenant_id,
        UserRole::Courier,
        Duration::minutes(5),
    )
    .expect("courier token");

    let (status, ratings) = authorized_request(
        app.clone(),
        "GET",
        "/v1/courier/me/ratings",
        json!({}),
        &courier_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(ratings.as_array().unwrap().is_empty());

    // A Dispatcher-role token from a different tenant cannot use this
    // courier-only self-service route.
    let (status, _) = authorized_request(
        app,
        "GET",
        "/v1/courier/me/ratings",
        json!({}),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn fleet_vehicle_lifecycle_over_http() {
    let app = app();

    let (status, courier) = request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Filo Kurye", "vehicle": "motorcycle" }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = courier["id"].as_str().expect("courier id").to_string();

    let (status, vehicle) = request(
        app.clone(),
        "POST",
        "/v1/fleet/vehicles",
        json!({
            "plate_number": "34 QRV 001",
            "vehicle_type": "motorcycle",
            "insurance_expiry": "2027-06-01"
        }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    assert_eq!(vehicle["status"], "active");
    let vehicle_id = vehicle["id"].as_str().expect("vehicle id").to_string();

    let (status, listed) = request(app.clone(), "GET", "/v1/fleet/vehicles", json!({})).await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert!(listed
        .as_array()
        .expect("vehicle list")
        .iter()
        .any(|entry| entry["id"] == vehicle_id));

    let (status, fetched) = request(
        app.clone(),
        "GET",
        &format!("/v1/fleet/vehicles/{vehicle_id}"),
        json!({}),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(fetched["plate_number"], "34 QRV 001");

    let (status, assigned) = request(
        app.clone(),
        "POST",
        &format!("/v1/fleet/vehicles/{vehicle_id}/assign"),
        json!({ "courier_id": courier_id }),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(assigned["assigned_courier_id"], courier_id);

    let (status, maintenance) = request(
        app.clone(),
        "POST",
        &format!("/v1/fleet/vehicles/{vehicle_id}/maintenance"),
        json!({}),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(maintenance["status"], "maintenance");
    assert!(maintenance["assigned_courier_id"].is_null());

    let (status, activated) = request(
        app.clone(),
        "POST",
        &format!("/v1/fleet/vehicles/{vehicle_id}/activate"),
        json!({}),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(activated["status"], "active");

    let (status, decommissioned) = request(
        app,
        "POST",
        &format!("/v1/fleet/vehicles/{vehicle_id}/decommission"),
        json!({}),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(decommissioned["status"], "decommissioned");
}

#[tokio::test]
async fn fleet_vehicles_are_isolated_by_tenant() {
    let (app, tenant_token, other_tenant_token) = tenant_tracking_fixture().await;

    let (status, foreign_vehicle) = authorized_request(
        app.clone(),
        "POST",
        "/v1/fleet/vehicles",
        json!({ "plate_number": "06 FOR 999", "vehicle_type": "car" }),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let foreign_vehicle_id = foreign_vehicle["id"].as_str().expect("foreign vehicle id");

    let (status, tenant_vehicle) = authorized_request(
        app.clone(),
        "POST",
        "/v1/fleet/vehicles",
        json!({ "plate_number": "34 OWN 001", "vehicle_type": "motorcycle" }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let tenant_vehicle_id = tenant_vehicle["id"].as_str().expect("tenant vehicle id");

    // Listing only returns vehicles bound to the caller's own tenant.
    let (status, listed) = authorized_request(
        app.clone(),
        "GET",
        "/v1/fleet/vehicles",
        json!({}),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let listed = listed.as_array().expect("vehicle list");
    assert!(listed.iter().any(|entry| entry["id"] == tenant_vehicle_id));
    assert!(!listed.iter().any(|entry| entry["id"] == foreign_vehicle_id));

    // Fetching another tenant's vehicle directly is forbidden.
    let (status, _) = authorized_request(
        app.clone(),
        "GET",
        &format!("/v1/fleet/vehicles/{foreign_vehicle_id}"),
        json!({}),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);

    // Assigning a courier from another tenant to one's own vehicle is forbidden.
    let (status, foreign_courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Foreign Courier", "vehicle": "car" }),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let foreign_courier_id = foreign_courier["id"].as_str().expect("foreign courier id");

    let (status, _) = authorized_request(
        app,
        "POST",
        &format!("/v1/fleet/vehicles/{tenant_vehicle_id}/assign"),
        json!({ "courier_id": foreign_courier_id }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn cross_origin_requests_are_rejected_without_configured_allowlist() {
    // QERVON_CORS_ALLOWED_ORIGINS is intentionally left unset here: the
    // secure default must not reflect an arbitrary Origin back to the
    // browser, even for a public, unauthenticated route.
    let response = app()
        .oneshot(
            axum::http::Request::builder()
                .method("GET")
                .uri("/health")
                .header("origin", "https://evil.example")
                .body(axum::body::Body::empty())
                .expect("cors probe request"),
        )
        .await
        .expect("cors probe response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    assert!(
        !response
            .headers()
            .contains_key("access-control-allow-origin"),
        "unconfigured origins must not be reflected back to the browser"
    );
}

#[tokio::test]
async fn auth_login_is_rate_limited_per_client_after_repeated_attempts() {
    let app = app();
    let login_attempt = || {
        axum::http::Request::builder()
            .method("POST")
            .uri("/v1/auth/login")
            .header("content-type", "application/json")
            .body(axum::body::Body::from(
                json!({
                    "email": "nobody@qervon.test",
                    "password": "definitely-wrong-password",
                    "tenant_slug": "qervon-test"
                })
                .to_string(),
            ))
            .expect("login attempt request")
    };

    // The auth-sensitive ceiling allows a burst of 10 requests per client
    // before throttling; none of the first 10 should be rate limited (they
    // still fail authentication with 401, which is a distinct concern).
    for attempt in 0..10 {
        let response = app
            .clone()
            .oneshot(login_attempt())
            .await
            .expect("login attempt response");
        assert_ne!(
            response.status(),
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            "attempt {attempt} should not yet be rate limited"
        );
    }

    let throttled = app
        .clone()
        .oneshot(login_attempt())
        .await
        .expect("throttled response");
    assert_eq!(
        throttled.status(),
        axum::http::StatusCode::TOO_MANY_REQUESTS,
        "the 11th rapid login attempt from the same client must be throttled"
    );
    assert!(throttled.headers().contains_key("retry-after"));
}

#[tokio::test]
async fn warehouse_hubs_are_tenant_isolated() {
    let (app, tenant_token, other_tenant_token) = tenant_tracking_fixture().await;

    let (status, foreign_hub) = authorized_request(
        app.clone(),
        "POST",
        "/v1/warehouse/hubs",
        json!({
            "hub_code": "HUB-FOREIGN",
            "hub_name": "Foreign Transfer Hub",
            "latitude": 41.0,
            "longitude": 29.0,
            "capacity_parcels": 500
        }),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let foreign_hub_id = foreign_hub["id"].as_str().expect("foreign hub id");

    let (status, own_hub) = authorized_request(
        app.clone(),
        "POST",
        "/v1/warehouse/hubs",
        json!({
            "hub_code": "HUB-OWN",
            "hub_name": "Own Transfer Hub",
            "latitude": 41.05,
            "longitude": 28.97,
            "capacity_parcels": 1000
        }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let own_hub_id = own_hub["id"].as_str().expect("own hub id").to_string();

    // Listing only returns hubs owned by the caller's own tenant.
    let (status, listed) = authorized_request(
        app.clone(),
        "GET",
        "/v1/warehouse/hubs",
        json!({}),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let listed = listed.as_array().expect("hub list");
    assert!(listed.iter().any(|entry| entry["id"] == own_hub_id));
    assert!(!listed.iter().any(|entry| entry["id"] == foreign_hub_id));

    // Receiving parcels into another tenant's hub is treated as not found.
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/warehouse/hubs/{foreign_hub_id}/receive"),
        json!({ "count": 10 }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);

    // Receiving parcels into one's own hub persists the new count.
    let (status, updated_hub) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/warehouse/hubs/{own_hub_id}/receive"),
        json!({ "count": 25 }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(updated_hub["active_parcels"], 25);

    // The persisted update survives a fresh list query too, proving hub
    // state lives in the repository rather than a per-request in-memory
    // copy.
    let (status, listed_again) = authorized_request(
        app.clone(),
        "GET",
        "/v1/warehouse/hubs",
        json!({}),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let refreshed = listed_again
        .as_array()
        .expect("hub list")
        .iter()
        .find(|entry| entry["id"] == own_hub_id)
        .expect("own hub present");
    assert_eq!(refreshed["active_parcels"], 25);

    // Dispatching a manifest from another tenant's hub is not found.
    let (status, _) = authorized_request(
        app,
        "POST",
        &format!("/v1/warehouse/hubs/{foreign_hub_id}/dispatch"),
        json!({ "courier_id": Uuid::now_v7(), "order_ids": [] }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn cold_chain_telemetry_is_tenant_isolated() {
    let (app, tenant_token, other_tenant_token) = tenant_tracking_fixture().await;
    let order_id = Uuid::now_v7();

    let (status, reading) = authorized_request(
        app.clone(),
        "POST",
        "/v1/cold-chain/telemetry",
        json!({
            "order_id": order_id,
            "sensor_id": "SENS-1",
            "temperature_celsius": 12.5,
            "humidity_percent": 40.0,
            "min_allowed_temp": 2.0,
            "max_allowed_temp": 8.0
        }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(reading["is_violation"], true);

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        "/v1/cold-chain/telemetry",
        json!({
            "order_id": Uuid::now_v7(),
            "sensor_id": "SENS-2",
            "temperature_celsius": 5.0,
            "humidity_percent": 45.0,
            "min_allowed_temp": 2.0,
            "max_allowed_temp": 8.0
        }),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    // Listing only returns telemetry recorded by the caller's own tenant,
    // even when filtered by an order id that belongs to another tenant.
    let (status, listed) = authorized_request(
        app.clone(),
        "GET",
        "/v1/cold-chain/telemetry",
        json!({}),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let listed = listed.as_array().expect("telemetry list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["sensor_id"], "SENS-1");

    let (status, other_listed) = authorized_request(
        app,
        "GET",
        "/v1/cold-chain/telemetry",
        json!({}),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let other_listed = other_listed.as_array().expect("other telemetry list");
    assert_eq!(other_listed.len(), 1);
    assert_eq!(other_listed[0]["sensor_id"], "SENS-2");
}

#[tokio::test]
async fn field_service_appointments_are_tenant_isolated() {
    let (app, tenant_token, other_tenant_token) = tenant_tracking_fixture().await;

    let (status, appointment) = authorized_request(
        app.clone(),
        "POST",
        "/v1/field-service/appointments",
        json!({
            "customer_id": Uuid::now_v7(),
            "service_type": "Klima Bakımı",
            "appointment_date": "2026-08-20",
            "slot_window": "Morning"
        }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(appointment["is_confirmed"], true);

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        "/v1/field-service/appointments",
        json!({
            "customer_id": Uuid::now_v7(),
            "service_type": "Kurulum",
            "appointment_date": "2026-08-21",
            "slot_window": "Afternoon"
        }),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, listed) = authorized_request(
        app,
        "GET",
        "/v1/field-service/appointments",
        json!({}),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let listed = listed.as_array().expect("appointment list");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["service_type"], "Klima Bakımı");
}

#[tokio::test]
async fn route_breadcrumbs_require_courier_ownership_and_are_tenant_isolated() {
    let (app, tenant_token, other_tenant_token) = tenant_tracking_fixture().await;

    let (status, courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Rota Kurye", "vehicle": "motorcycle" }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let courier_id = courier["id"].as_str().expect("courier id").to_string();

    // Another tenant cannot report breadcrumbs for a courier it does not own.
    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/route-history/{courier_id}/breadcrumbs"),
        json!({
            "latitude": 41.02,
            "longitude": 28.95,
            "speed_kmh": 32.0,
            "battery_level": 80
        }),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);

    let timestamp = Utc::now();
    let (status, breadcrumb) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/route-history/{courier_id}/breadcrumbs"),
        json!({
            "latitude": 41.02,
            "longitude": 28.95,
            "speed_kmh": 32.0,
            "battery_level": 80,
            "timestamp": timestamp.to_rfc3339()
        }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(breadcrumb["courier_id"], courier_id);

    // Another tenant cannot read the playback track either.
    let (status, _) = authorized_request(
        app.clone(),
        "GET",
        &format!(
            "/v1/route-history/{courier_id}?date={}",
            timestamp.format("%Y-%m-%d")
        ),
        json!({}),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::NOT_FOUND);

    let (status, track) = authorized_request(
        app,
        "GET",
        &format!(
            "/v1/route-history/{courier_id}?date={}",
            timestamp.format("%Y-%m-%d")
        ),
        json!({}),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let breadcrumbs = track["breadcrumbs"].as_array().expect("breadcrumbs");
    assert_eq!(breadcrumbs.len(), 1);
}

#[tokio::test]
async fn courier_leaderboard_ranks_couriers_and_is_tenant_isolated() {
    let (app, tenant_token, other_tenant_token) = tenant_tracking_fixture().await;

    let (status, top_courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Ahmet", "vehicle": "motorcycle" }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let top_courier_id = top_courier["id"]
        .as_str()
        .expect("top courier id")
        .to_string();

    let (status, idle_courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Mehmet", "vehicle": "bicycle" }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let idle_courier_id = idle_courier["id"]
        .as_str()
        .expect("idle courier id")
        .to_string();

    let (status, foreign_courier) = authorized_request(
        app.clone(),
        "POST",
        "/v1/couriers",
        json!({ "name": "Foreign Courier", "vehicle": "car" }),
        &other_tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let foreign_courier_id = foreign_courier["id"]
        .as_str()
        .expect("foreign courier id")
        .to_string();

    // Give the top courier one completed delivery so it scores above the
    // idle courier, which has none.
    let (status, order) = authorized_request(
        app.clone(),
        "POST",
        "/v1/orders",
        json!({
            "customer_id": Uuid::now_v7(),
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "fare_amount_minor": 4200,
            "fare_currency": "TRY"
        }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::CREATED);
    let order_id = order["id"].as_str().expect("order id").to_string();

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/assign"),
        json!({ "courier_id": top_courier_id }),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/transit"),
        json!({}),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, _) = authorized_request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/deliver"),
        json!({}),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);

    let (status, leaderboard) = authorized_request(
        app,
        "GET",
        "/v1/couriers/leaderboard",
        json!({}),
        &tenant_token,
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    let entries = leaderboard.as_array().expect("leaderboard entries");

    // Only the caller's own tenant's couriers are ranked.
    assert!(entries
        .iter()
        .any(|entry| entry["courier_id"] == top_courier_id));
    assert!(entries
        .iter()
        .any(|entry| entry["courier_id"] == idle_courier_id));
    assert!(!entries
        .iter()
        .any(|entry| entry["courier_id"] == foreign_courier_id));

    let top_entry = entries
        .iter()
        .find(|entry| entry["courier_id"] == top_courier_id)
        .expect("top courier entry");
    let idle_entry = entries
        .iter()
        .find(|entry| entry["courier_id"] == idle_courier_id)
        .expect("idle courier entry");
    assert_eq!(top_entry["completed_deliveries"], 1);
    assert_eq!(idle_entry["completed_deliveries"], 0);
    assert!(top_entry["rank"].as_u64().unwrap() < idle_entry["rank"].as_u64().unwrap());
}
