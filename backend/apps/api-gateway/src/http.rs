// =============================================================================
// File:           backend/apps/api-gateway/src/http.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   HTTP router and handlers for the Qervon delivery vertical slice.
//
// Specification:
//   QAS-000005, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use aes_gcm::{
    aead::{Aead, OsRng},
    AeadCore, Aes256Gcm, KeyInit,
};
use axum::{
    extract::{DefaultBodyLimit, Extension, Path, Request, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{Duration, Utc};
use qervon_api_contracts::{
    AddressDto, AssignCourierRequest, CourierResponse, CreateCustomerOrderRequest,
    CreateOrderRequest, OperationsOverviewResponse, OrderResponse, RegisterCourierRequest,
    SetCourierAvailabilityRequest, UpdateLocationRequest,
};
use qervon_application::{
    CreateInvoiceInput, CreateOrderInput, RegisterCourierInput, SendNotificationInput,
};
use qervon_domain::{
    Address, Location, Money, NotificationChannel, OrderId, RefreshSession, TenantId, UserId,
    UserRole, VehicleType,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::Instant;

use crate::api_error::ApiError;
use crate::auth::{
    hash_refresh_token, issue_access_token, new_refresh_token, verify_access_token, AccessClaims,
};
use crate::state::AppState;

async fn serve_swagger_ui() -> axum::response::Html<&'static str> {
    axum::response::Html(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Qervon LOS — Swagger UI</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js"></script>
  <script>
    SwaggerUIBundle({
      url: '/api-docs/openapi.json',
      dom_id: '#swagger-ui',
    });
  </script>
</body>
</html>"#,
    )
}

async fn serve_openapi_spec() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "openapi": "3.0.3",
        "info": {
            "title": "Qervon Logistics Operating System (LOS) API",
            "version": "0.1.0",
            "description": "High-performance, modular multi-tenant logistics & dispatch API"
        },
        "paths": {
            "/v1/orders": {
                "post": { "summary": "Create Order" },
                "get": { "summary": "List Orders" }
            },
            "/v1/couriers": {
                "post": { "summary": "Register Courier" },
                "get": { "summary": "List Couriers" }
            },
            "/v1/couriers/{id}/location": {
                "post": { "summary": "Update Courier GPS Location" }
            },
            "/v1/orders/{id}/assign": {
                "post": { "summary": "AI Dispatch Courier Assignment" }
            },
            "/ws/tracking": {
                "get": { "summary": "Tenant-scoped WebSocket real-time location stream" }
            },
            "/v1/tracking/live": {
                "get": { "summary": "List latest courier locations for the authenticated tenant" }
            },
            "/v1/orders/{id}/tracking": {
                "get": { "summary": "Track the assigned courier for one authorized order" }
            }
        }
    }))
}

async fn serve_mobile_customer() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../static/mobile-customer.html"))
}

async fn serve_web_manifest() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/manifest+json")],
        include_str!("../static/manifest.webmanifest"),
    )
}

async fn serve_service_worker() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "application/javascript")],
        include_str!("../static/sw.js"),
    )
}

async fn serve_mobile_courier() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../static/mobile-courier.html"))
}

pub fn router(state: AppState) -> Router {
    let public = Router::new()
        .route("/", get(serve_dashboard))
        .route("/customer", get(serve_customer_portal))
        .route("/login", get(serve_login))
        .route("/mobile-customer", get(serve_mobile_customer))
        .route("/mobile-courier", get(serve_mobile_courier))
        .route("/manifest.webmanifest", get(serve_web_manifest))
        .route("/sw.js", get(serve_service_worker))
        .route("/swagger-ui", get(serve_swagger_ui))
        .route("/api-docs/openapi.json", get(serve_openapi_spec))
        .route("/health", get(health))
        .route("/ready", get(readiness))
        .route("/metrics", get(metrics_handler));
    let public = public
        .route("/v1/auth/register", post(auth_register))
        .route("/v1/auth/login", post(auth_login))
        .route("/v1/auth/refresh", post(auth_refresh))
        .route("/v1/auth/logout", post(auth_logout));
    let public = public
        .route("/v1/browser/auth/login", post(browser_login))
        .route("/v1/browser/auth/refresh", post(browser_refresh))
        .route("/v1/browser/auth/logout", post(browser_logout));

    let operations = Router::new()
        .route("/v1/operations/overview", get(operations_overview))
        .route("/v1/users", post(register_user))
        .route("/v1/couriers", post(register_courier).get(list_couriers))
        .route("/v1/orders", post(create_order).get(list_orders))
        .route("/v1/orders/{id}", get(get_order))
        .route("/v1/orders/{id}/assign", post(assign_courier))
        .route("/v1/orders/{id}/transit", post(start_transit))
        .route("/v1/orders/{id}/deliver", post(deliver_order))
        .route("/v1/orders/{id}/cancel", post(cancel_order))
        .route_layer(middleware::from_fn(require_operational_access));
    let customer_operations = Router::new()
        .route("/v1/customer/profile", get(get_customer_profile))
        .route("/v1/customer/profile/addresses", post(add_customer_address))
        .route(
            "/v1/customer/profile/addresses/{id}",
            delete(remove_customer_address),
        )
        .route(
            "/v1/customer/orders",
            post(create_customer_order).get(list_customer_orders),
        )
        .route(
            "/v1/customer/orders/{id}/invoice",
            get(get_customer_order_invoice),
        )
        .route(
            "/v1/customer/orders/{id}/proof-of-delivery",
            get(get_customer_order_proof),
        )
        .route(
            "/v1/customer/notifications",
            get(list_customer_notifications),
        )
        .route(
            "/v1/customer/webhooks",
            post(create_customer_webhook).get(list_customer_webhooks),
        )
        .route(
            "/v1/customer/webhooks/{id}",
            delete(delete_customer_webhook),
        )
        .route_layer(middleware::from_fn(require_customer_access));
    let location_publisher = Router::new()
        .route("/v1/couriers/{id}/location", post(update_courier_location))
        .route_layer(middleware::from_fn(require_location_publisher));
    let courier_operations = Router::new()
        .route("/v1/courier/me", get(get_own_courier))
        .route("/v1/courier/me/status", post(set_own_courier_availability))
        .route("/v1/courier/me/location", post(update_own_courier_location))
        .route("/v1/courier/orders", get(list_courier_orders))
        .route(
            "/v1/courier/orders/{id}/pickup",
            post(courier_start_transit),
        )
        .route(
            "/v1/courier/orders/{id}/deliver",
            post(courier_deliver_order),
        )
        .route_layer(middleware::from_fn(require_courier_access));
    let tracking_consumers = Router::new()
        .route("/v1/tracking/live", get(list_live_locations))
        .route("/v1/orders/{id}/tracking", get(order_tracking))
        .route("/ws/tracking", get(ws_tracking_handler))
        .route_layer(middleware::from_fn(require_tracking_consumer));
    let push_operations = Router::new()
        .route("/v1/push/config", get(get_push_config))
        .route(
            "/v1/push/subscriptions",
            post(upsert_push_subscription).delete(delete_push_subscription),
        )
        .route_layer(middleware::from_fn(require_signed_user));
    // Location events currently lack a tenant key in the delivery aggregate.
    // Do not expose an all-tenant stream to signed end users until the event
    // and assignment models carry that boundary end-to-end.
    let protected = operations
        .merge(location_publisher)
        .merge(courier_operations)
        .merge(customer_operations)
        .merge(tracking_consumers)
        .merge(push_operations)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_api_access,
        ));

    public
        .merge(protected)
        .with_state(state.clone())
        .layer(DefaultBodyLimit::max(1_048_576))
        .layer(middleware::from_fn_with_state(state, observe_request))
}

async fn observe_request(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let started = Instant::now();
    let method = request.method().clone();
    let request_id = uuid::Uuid::now_v7().to_string();
    let mut response = next.run(request).await;
    let status = response.status();
    let elapsed = started.elapsed();
    state.runtime_metrics.observe(status.as_u16(), elapsed);

    response.headers_mut().insert(
        HeaderName::from_static("x-request-id"),
        HeaderValue::from_str(&request_id).unwrap_or_else(|_| HeaderValue::from_static("invalid")),
    );
    response.headers_mut().insert(
        HeaderName::from_static("x-content-type-options"),
        HeaderValue::from_static("nosniff"),
    );
    response.headers_mut().insert(
        HeaderName::from_static("referrer-policy"),
        HeaderValue::from_static("no-referrer"),
    );
    tracing::info!(
        request_id = %request_id,
        method = %method,
        status = status.as_u16(),
        duration_ms = elapsed.as_secs_f64() * 1_000.0,
        "HTTP request completed"
    );
    response
}

#[derive(Clone, Copy)]
struct ServiceAccess;

async fn require_api_access(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if state.api_access_token.is_none() && state.token_signing_secret.is_none() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({"status": 503, "title": "Service Unavailable", "detail": "API authentication is not configured"})),
        )
            .into_response();
    }
    let bearer_token = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "));
    let cookie_token = cookie_value(request.headers(), "qervon_access_token");
    let supplied_token = bearer_token.or(cookie_token);
    if bearer_token.is_none()
        && cookie_token.is_some()
        && !matches!(
            *request.method(),
            Method::GET | Method::HEAD | Method::OPTIONS
        )
        && !csrf_is_valid(request.headers())
    {
        return forbidden_response("CSRF validation failed").into_response();
    }
    if state
        .api_access_token
        .as_deref()
        .is_some_and(|expected_token| supplied_token.is_some_and(|value| value == expected_token))
    {
        let mut request = request;
        request.extensions_mut().insert(ServiceAccess);
        next.run(request).await
    } else if let (Some(secret), Some(token)) =
        (state.token_signing_secret.as_deref(), supplied_token)
    {
        match verify_access_token(secret.as_bytes(), token) {
            Ok(claims) => {
                let mut request = request;
                request.extensions_mut().insert(claims);
                next.run(request).await
            }
            Err(_) => unauthorized_response(),
        }
    } else {
        unauthorized_response()
    }
}

async fn require_operational_access(request: Request, next: Next) -> Response {
    if request.extensions().get::<ServiceAccess>().is_some() {
        return next.run(request).await;
    }
    let Some(claims) = request.extensions().get::<AccessClaims>() else {
        return unauthorized_response();
    };
    let can_operate = matches!(
        claims.role,
        UserRole::SuperAdmin
            | UserRole::Admin
            | UserRole::Operator
            | UserRole::Dispatcher
            | UserRole::FleetManager
    );
    if can_operate {
        next.run(request).await
    } else {
        (
            StatusCode::FORBIDDEN,
            Json(json!({"status": 403, "title": "Forbidden", "detail": "this role cannot access operational routes"})),
        )
            .into_response()
    }
}

async fn require_location_publisher(request: Request, next: Next) -> Response {
    if request.extensions().get::<ServiceAccess>().is_some() {
        return next.run(request).await;
    }
    let Some(claims) = request.extensions().get::<AccessClaims>() else {
        return unauthorized_response();
    };
    if matches!(
        claims.role,
        UserRole::Courier
            | UserRole::SuperAdmin
            | UserRole::Admin
            | UserRole::Operator
            | UserRole::Dispatcher
    ) {
        next.run(request).await
    } else {
        forbidden_response("this role cannot publish courier location").into_response()
    }
}

async fn require_courier_access(request: Request, next: Next) -> Response {
    let Some(claims) = request.extensions().get::<AccessClaims>() else {
        return unauthorized_response();
    };
    if claims.role == UserRole::Courier {
        next.run(request).await
    } else {
        forbidden_response("this route is only available to courier sessions").into_response()
    }
}

async fn require_customer_access(request: Request, next: Next) -> Response {
    let Some(claims) = request.extensions().get::<AccessClaims>() else {
        return unauthorized_response();
    };
    if claims.role == UserRole::Customer {
        next.run(request).await
    } else {
        forbidden_response("this route is only available to customer sessions").into_response()
    }
}

async fn require_tracking_consumer(request: Request, next: Next) -> Response {
    let Some(claims) = request.extensions().get::<AccessClaims>() else {
        return unauthorized_response();
    };
    if matches!(
        claims.role,
        UserRole::Customer
            | UserRole::SuperAdmin
            | UserRole::Admin
            | UserRole::Operator
            | UserRole::Dispatcher
            | UserRole::Support
    ) {
        next.run(request).await
    } else {
        forbidden_response("this role cannot view live tracking").into_response()
    }
}

async fn require_signed_user(request: Request, next: Next) -> Response {
    if request.extensions().get::<AccessClaims>().is_some() {
        next.run(request).await
    } else {
        forbidden_response("a signed user session is required").into_response()
    }
}

fn unauthorized_response() -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(json!({"status": 401, "title": "Unauthorized", "detail": "a valid bearer token is required"})),
    )
        .into_response()
}

fn forbidden_response(detail: &'static str) -> (StatusCode, Json<Value>) {
    (
        StatusCode::FORBIDDEN,
        Json(json!({"status": 403, "title": "Forbidden", "detail": detail})),
    )
}

#[derive(Deserialize)]
struct AuthRegisterRequest {
    email: String,
    display_name: String,
    password: String,
}

#[derive(Deserialize)]
struct LoginRequest {
    email: String,
    password: String,
    tenant_slug: String,
}

#[derive(Deserialize)]
struct RefreshRequest {
    refresh_token: String,
}

#[derive(Serialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    token_type: &'static str,
    expires_in_seconds: i64,
}

#[derive(Deserialize)]
struct BrowserLoginRequest {
    email: String,
    password: String,
    tenant_slug: String,
}

#[derive(Deserialize)]
struct CompleteDeliveryRequest {
    recipient_name: String,
    qr_barcode_verified: bool,
    digital_signature_base64: Option<String>,
    photo_evidence_url: Option<String>,
}

#[derive(Deserialize)]
struct CreateCustomerAddressRequest {
    label: String,
    latitude: f64,
    longitude: f64,
    full_address: String,
}

async fn browser_login(
    State(state): State<AppState>,
    Json(request): Json<BrowserLoginRequest>,
) -> Result<(HeaderMap, StatusCode), ApiError> {
    let user = state
        .auth
        .authenticate(&request.email, &request.password)
        .await
        .map_err(|_| invalid_credentials())?;
    let tenant = state
        .tenants
        .find_by_slug(&request.tenant_slug)
        .await?
        .ok_or_else(invalid_credentials)?;
    if state
        .tenants
        .find_membership(tenant.id, user.id)
        .await?
        .is_none()
    {
        return Err(invalid_credentials());
    }
    let session = issue_session(&state, user.id, tenant.id, user.role).await?;
    Ok((browser_session_headers(&session), StatusCode::NO_CONTENT))
}

async fn auth_register(
    State(state): State<AppState>,
    Json(request): Json<AuthRegisterRequest>,
) -> Result<StatusCode, ApiError> {
    // A public caller can only become a customer. Tenant membership and every
    // operational role are provisioned by an authenticated tenant administrator.
    let user = state
        .auth
        .register(
            request.email,
            request.display_name,
            request.password,
            UserRole::Customer,
        )
        .await?;
    state.customers.create_profile(user.id).await?;
    Ok(StatusCode::CREATED)
}

async fn auth_login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let user = state
        .auth
        .authenticate(&request.email, &request.password)
        .await
        .map_err(|_| invalid_credentials())?;
    let tenant = state
        .tenants
        .find_by_slug(&request.tenant_slug)
        .await?
        .ok_or_else(invalid_credentials)?;
    if state
        .tenants
        .find_membership(tenant.id, user.id)
        .await?
        .is_none()
    {
        return Err(invalid_credentials());
    }
    Ok(Json(
        issue_session(&state, user.id, tenant.id, user.role).await?,
    ))
}

async fn auth_refresh(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let session = state
        .credentials
        .find_refresh_session(&hash_refresh_token(&request.refresh_token))
        .await?
        .filter(|session| session.is_active_at(Utc::now()))
        .ok_or_else(invalid_credentials)?;
    let user = state
        .identity
        .get_user(session.user_id)
        .await
        .map_err(|_| invalid_credentials())?;
    if !user.is_active()
        || state
            .tenants
            .find_membership(session.tenant_id, user.id)
            .await?
            .is_none()
    {
        return Err(invalid_credentials());
    }
    state.auth.revoke_refresh_session(session.id).await?;
    Ok(Json(
        issue_session(&state, user.id, session.tenant_id, user.role).await?,
    ))
}

async fn auth_logout(
    State(state): State<AppState>,
    Json(request): Json<RefreshRequest>,
) -> StatusCode {
    // Logout is deliberately idempotent and does not disclose whether a token existed.
    if let Ok(Some(session)) = state
        .credentials
        .find_refresh_session(&hash_refresh_token(&request.refresh_token))
        .await
    {
        let _ = state.auth.revoke_refresh_session(session.id).await;
    }
    StatusCode::NO_CONTENT
}

async fn browser_refresh(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, StatusCode), ApiError> {
    if !csrf_is_valid(&headers) {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "CSRF validation failed".into(),
        });
    }
    let refresh_token =
        cookie_value(&headers, "qervon_refresh_token").ok_or_else(invalid_credentials)?;
    let session = state
        .credentials
        .find_refresh_session(&hash_refresh_token(refresh_token))
        .await?
        .filter(|session| session.is_active_at(Utc::now()))
        .ok_or_else(invalid_credentials)?;
    let user = state
        .identity
        .get_user(session.user_id)
        .await
        .map_err(|_| invalid_credentials())?;
    if !user.is_active()
        || state
            .tenants
            .find_membership(session.tenant_id, user.id)
            .await?
            .is_none()
    {
        return Err(invalid_credentials());
    }
    state.auth.revoke_refresh_session(session.id).await?;
    let next = issue_session(&state, user.id, session.tenant_id, user.role).await?;
    Ok((browser_session_headers(&next), StatusCode::NO_CONTENT))
}

async fn browser_logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<(HeaderMap, StatusCode), ApiError> {
    if !csrf_is_valid(&headers) {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "CSRF validation failed".into(),
        });
    }
    if let Some(token) = cookie_value(&headers, "qervon_refresh_token") {
        if let Ok(Some(session)) = state
            .credentials
            .find_refresh_session(&hash_refresh_token(token))
            .await
        {
            let _ = state.auth.revoke_refresh_session(session.id).await;
        }
    }
    Ok((expired_browser_session_headers(), StatusCode::NO_CONTENT))
}

fn browser_session_headers(session: &AuthResponse) -> HeaderMap {
    let csrf = new_refresh_token();
    let mut headers = HeaderMap::new();
    let access = format!(
        "qervon_access_token={}; Path=/; Max-Age={}; HttpOnly; SameSite=Lax; Secure",
        session.access_token, session.expires_in_seconds
    );
    let refresh = format!("qervon_refresh_token={}; Path=/v1/browser/auth; Max-Age={}; HttpOnly; SameSite=Strict; Secure", session.refresh_token, 60 * 60 * 24 * 30);
    let csrf_cookie = format!(
        "qervon_csrf_token={csrf}; Path=/; Max-Age={}; SameSite=Strict; Secure",
        60 * 60 * 24 * 30
    );
    for value in [access, refresh, csrf_cookie] {
        headers.append(
            header::SET_COOKIE,
            HeaderValue::from_str(&value).expect("cookie header is valid"),
        );
    }
    headers
}

fn expired_browser_session_headers() -> HeaderMap {
    let mut headers = HeaderMap::new();
    for value in [
        "qervon_access_token=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax; Secure",
        "qervon_refresh_token=; Path=/v1/browser/auth; Max-Age=0; HttpOnly; SameSite=Strict; Secure",
        "qervon_csrf_token=; Path=/; Max-Age=0; SameSite=Strict; Secure",
    ] {
        headers.append(header::SET_COOKIE, HeaderValue::from_static(value));
    }
    headers
}

fn cookie_value<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(header::COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .find_map(|part| {
            let (key, value) = part.trim().split_once('=')?;
            (key == name).then_some(value)
        })
}

fn csrf_is_valid(headers: &HeaderMap) -> bool {
    let cookie = cookie_value(headers, "qervon_csrf_token");
    let supplied = headers
        .get("x-csrf-token")
        .and_then(|value| value.to_str().ok());
    cookie.is_some() && cookie == supplied
}

async fn issue_session(
    state: &AppState,
    user_id: UserId,
    tenant_id: TenantId,
    role: UserRole,
) -> Result<AuthResponse, ApiError> {
    let secret = state
        .token_signing_secret
        .as_deref()
        .ok_or_else(|| ApiError {
            status: StatusCode::SERVICE_UNAVAILABLE,
            detail: "interactive authentication is not configured".into(),
        })?;
    let access_lifetime = Duration::minutes(15);
    let access_token = issue_access_token(
        secret.as_bytes(),
        user_id.0,
        tenant_id.0,
        role,
        access_lifetime,
    )
    .map_err(ApiError::unprocessable)?;
    let refresh_token = new_refresh_token();
    let now = Utc::now();
    state
        .credentials
        .save_refresh_session(&RefreshSession {
            id: uuid::Uuid::now_v7(),
            user_id,
            tenant_id,
            token_hash: hash_refresh_token(&refresh_token),
            expires_at: now + Duration::days(30),
            revoked_at: None,
            created_at: now,
        })
        .await?;
    Ok(AuthResponse {
        access_token,
        refresh_token,
        token_type: "Bearer",
        expires_in_seconds: access_lifetime.num_seconds(),
    })
}

fn invalid_credentials() -> ApiError {
    ApiError {
        status: StatusCode::UNAUTHORIZED,
        detail: "invalid credentials or tenant access".into(),
    }
}

async fn serve_dashboard() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../static/index.html"))
}

async fn serve_customer_portal() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../static/customer.html"))
}

async fn serve_login() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../static/login.html"))
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn readiness(State(state): State<AppState>) -> Response {
    let authentication_configured =
        state.api_access_token.is_some() || state.token_signing_secret.is_some();
    let body = Json(json!({
        "status": if authentication_configured { "ready" } else { "not_ready" },
        "storage": state.storage_backend.as_str(),
        "authentication_configured": authentication_configured,
    }));
    if authentication_configured {
        (StatusCode::OK, body).into_response()
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, body).into_response()
    }
}

async fn metrics_handler(State(state): State<AppState>) -> String {
    let live_locations = state
        .latest_locations
        .read()
        .map(|locations| locations.len())
        .unwrap_or_default();
    let metrics = state.runtime_metrics.snapshot();
    let uptime_seconds = state.started_at.elapsed().as_secs_f64();
    format!(
        "# HELP qervon_live_courier_locations Current courier locations held by this API process\n\
         # TYPE qervon_live_courier_locations gauge\n\
         qervon_live_courier_locations {live_locations}\n\
         # HELP qervon_auth_configured Whether an API authentication method is configured\n\
         # TYPE qervon_auth_configured gauge\n\
         qervon_auth_configured {}\n\
         # HELP qervon_http_requests_total HTTP responses completed by status class\n\
         # TYPE qervon_http_requests_total counter\n\
         qervon_http_requests_total{{status_class=\"2xx\"}} {}\n\
         qervon_http_requests_total{{status_class=\"3xx\"}} {}\n\
         qervon_http_requests_total{{status_class=\"4xx\"}} {}\n\
         qervon_http_requests_total{{status_class=\"5xx\"}} {}\n\
         qervon_http_requests_total{{status_class=\"other\"}} {}\n\
         # HELP qervon_http_response_duration_microseconds_total Total completed HTTP response time\n\
         # TYPE qervon_http_response_duration_microseconds_total counter\n\
         qervon_http_response_duration_microseconds_total {}\n\
         # HELP qervon_process_uptime_seconds API process uptime\n\
         # TYPE qervon_process_uptime_seconds gauge\n\
         qervon_process_uptime_seconds {:.3}\n",
        u8::from(state.api_access_token.is_some() || state.token_signing_secret.is_some()),
        metrics.responses_2xx,
        metrics.responses_3xx,
        metrics.responses_4xx,
        metrics.responses_5xx,
        metrics.responses_other,
        metrics.duration_microseconds,
        uptime_seconds,
    )
}

async fn ws_tracking_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(move |mut socket| async move {
        let mut rx = state.location_tx.subscribe();
        while let Ok(msg) = rx.recv().await {
            if msg.tenant_id == claims.tenant_id {
                if let Ok(json) = serde_json::to_string(&msg) {
                    if socket
                        .send(axum::extract::ws::Message::Text(json.into()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
            }
        }
    })
}

async fn register_user(
    State(state): State<AppState>,
    Json(request): Json<qervon_api_contracts::RegisterUserRequest>,
) -> Result<(StatusCode, Json<qervon_api_contracts::UserResponse>), ApiError> {
    let role: qervon_domain::UserRole = request
        .role
        .parse()
        .map_err(|_| ApiError::unprocessable("invalid user role"))?;
    let user = state
        .identity
        .register_user(qervon_application::CreateUserInput {
            email: request.email,
            display_name: request.display_name,
            role,
        })
        .await?;
    Ok((StatusCode::CREATED, Json((&user).into())))
}

async fn register_courier(
    State(state): State<AppState>,
    claims: Option<Extension<AccessClaims>>,
    Json(request): Json<RegisterCourierRequest>,
) -> Result<(StatusCode, Json<CourierResponse>), ApiError> {
    let vehicle: VehicleType = request
        .vehicle
        .parse()
        .map_err(|_| ApiError::unprocessable("invalid vehicle type"))?;
    let courier = state
        .couriers
        .register_courier(RegisterCourierInput {
            id: request.id.unwrap_or_else(uuid::Uuid::now_v7),
            name: request.name,
            vehicle,
        })
        .await?;
    if let Some(Extension(claims)) = claims {
        state
            .tenants
            .bind_courier(TenantId(claims.tenant_id), courier.id)
            .await?;
    }
    Ok((StatusCode::CREATED, Json((&courier).into())))
}

async fn list_couriers(
    State(state): State<AppState>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<Vec<CourierResponse>>, ApiError> {
    let couriers = state.couriers.list_all_couriers().await?;
    let mut response = Vec::new();
    for courier in couriers {
        if let Some(Extension(claims)) = &claims {
            if state.tenants.find_courier_tenant(courier.id).await?
                != Some(TenantId(claims.tenant_id))
            {
                continue;
            }
        }
        response.push(CourierResponse::from(&courier));
    }
    Ok(Json(response))
}

async fn update_courier_location(
    State(state): State<AppState>,
    Path(courier_id): Path<uuid::Uuid>,
    claims: Option<Extension<AccessClaims>>,
    Json(request): Json<UpdateLocationRequest>,
) -> Result<Json<CourierResponse>, ApiError> {
    let tenant_id = if let Some(Extension(claims)) = &claims {
        let tenant_id = TenantId(claims.tenant_id);
        let resource_tenant = state
            .tenants
            .find_courier_tenant(courier_id)
            .await?
            .ok_or_else(|| ApiError::unprocessable("courier has no tenant ownership"))?;
        if resource_tenant != tenant_id
            || (claims.role == UserRole::Courier && claims.subject != courier_id)
        {
            return Err(ApiError {
                status: StatusCode::FORBIDDEN,
                detail: "courier location ownership check failed".into(),
            });
        }
        tenant_id.0
    } else {
        uuid::Uuid::nil()
    };
    persist_courier_location(&state, courier_id, tenant_id, request).await
}

async fn persist_courier_location(
    state: &AppState,
    courier_id: uuid::Uuid,
    tenant_id: uuid::Uuid,
    request: UpdateLocationRequest,
) -> Result<Json<CourierResponse>, ApiError> {
    let location = Location::new(request.latitude, request.longitude)?;
    state
        .tracking
        .record_location(courier_id, location, request.speed_kmh, request.battery_pct)
        .await?;
    let courier = state
        .couriers
        .update_courier_location(courier_id, location)
        .await?;

    // Broadcast live location event over WebSocket channel
    let event = crate::state::LocationUpdateEvent {
        courier_id,
        tenant_id,
        latitude: request.latitude,
        longitude: request.longitude,
        timestamp: chrono::Utc::now(),
    };
    state.publish_location(event).await.map_err(|error| {
        ApiError::unprocessable(format!("could not relay courier location: {error}"))
    })?;

    Ok(Json((&courier).into()))
}

async fn get_own_courier(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<CourierResponse>, ApiError> {
    require_courier_subject(&state, &claims).await?;
    let courier = state.couriers.get_courier(claims.subject).await?;
    Ok(Json((&courier).into()))
}

async fn set_own_courier_availability(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<SetCourierAvailabilityRequest>,
) -> Result<Json<CourierResponse>, ApiError> {
    require_courier_subject(&state, &claims).await?;
    let courier = state
        .couriers
        .set_courier_online_status(claims.subject, request.online)
        .await?;
    Ok(Json((&courier).into()))
}

async fn update_own_courier_location(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<UpdateLocationRequest>,
) -> Result<Json<CourierResponse>, ApiError> {
    require_courier_subject(&state, &claims).await?;
    persist_courier_location(&state, claims.subject, claims.tenant_id, request).await
}

async fn list_courier_orders(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<OrderResponse>>, ApiError> {
    require_courier_subject(&state, &claims).await?;
    let mut response = Vec::new();
    for order in state.orders.list_all().await? {
        if order.assigned_courier_id != Some(claims.subject)
            || matches!(
                order.status,
                qervon_domain::OrderStatus::Delivered | qervon_domain::OrderStatus::Cancelled
            )
        {
            continue;
        }
        if state.tenants.find_order_tenant(order.id).await? == Some(TenantId(claims.tenant_id)) {
            response.push((&order).into());
        }
    }
    Ok(Json(response))
}

async fn courier_start_transit(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<OrderResponse>, ApiError> {
    let order_id = OrderId(order_id);
    require_courier_order(&state, order_id, &claims).await?;
    let order = state.dispatch.start_transit(order_id).await?;
    Ok(Json((&order).into()))
}

async fn courier_deliver_order(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<CompleteDeliveryRequest>,
) -> Result<Json<OrderResponse>, ApiError> {
    let order_id = OrderId(order_id);
    require_courier_order(&state, order_id, &claims).await?;
    if !request.qr_barcode_verified
        && request.digital_signature_base64.is_none()
        && request.photo_evidence_url.is_none()
    {
        return Err(ApiError::unprocessable(
            "QR verification, signature, or photo evidence is required",
        ));
    }
    let proof = qervon_domain::ProofOfDeliveryRecord::new(
        order_id.0,
        claims.subject,
        request.recipient_name,
        request.qr_barcode_verified,
        request.digital_signature_base64,
        request.photo_evidence_url,
    )?;
    let order = if state.postgres_pool.is_some() {
        complete_delivery_atomically(&state, order_id, &proof, claims.tenant_id).await?
    } else {
        let order = state.dispatch.deliver_order(order_id).await?;
        state.proofs_of_delivery.create(&proof).await?;
        create_delivery_financial_records(&state, &order).await?;
        enqueue_delivery_outbox_event(&state, &order, claims.tenant_id).await?;
        order
    };
    Ok(Json((&order).into()))
}

/// Commits every durable delivery side effect, including the webhook outbox row,
/// together. A worker can therefore never observe an event for a delivery that
/// was rolled back, and a delivered order can never lose its event on a crash.
async fn complete_delivery_atomically(
    state: &AppState,
    order_id: OrderId,
    proof: &qervon_domain::ProofOfDeliveryRecord,
    tenant_id: uuid::Uuid,
) -> Result<qervon_domain::Order, ApiError> {
    let pool = state
        .postgres_pool
        .as_ref()
        .expect("PostgreSQL pool checked before atomic delivery");
    let mut order = state.orders.get_order(order_id).await?;
    order.deliver(Utc::now())?;
    let courier_id = order
        .assigned_courier_id
        .ok_or_else(|| ApiError::unprocessable("delivered order has no assigned courier"))?;
    let mut transaction = pool.begin().await.map_err(|error| {
        ApiError::unprocessable(format!("could not start delivery transaction: {error}"))
    })?;

    sqlx::query(
        "UPDATE orders.orders SET status = 'delivered', delivered_at = $2 WHERE id = $1 AND status = 'in_transit'",
    )
    .bind(order.id.0)
    .bind(order.delivered_at)
    .execute(&mut *transaction)
    .await
    .map_err(|error| ApiError::unprocessable(format!("could not complete order: {error}")))?
    .rows_affected()
    .eq(&1)
    .then_some(())
    .ok_or_else(|| ApiError::unprocessable("order is no longer in transit"))?;

    sqlx::query("UPDATE couriers.couriers SET status = 'available' WHERE id = $1")
        .bind(courier_id)
        .execute(&mut *transaction)
        .await
        .map_err(|error| ApiError::unprocessable(format!("could not release courier: {error}")))?;
    sqlx::query("UPDATE dispatch.assignments SET status = 'completed' WHERE order_id = $1 AND status = 'assigned'")
        .bind(order.id.0)
        .execute(&mut *transaction)
        .await
        .map_err(|error| ApiError::unprocessable(format!("could not complete assignment: {error}")))?;
    sqlx::query(
        "INSERT INTO delivery.proofs_of_delivery (id, order_id, courier_id, recipient_name, qr_barcode_verified, digital_signature_base64, photo_evidence_url, delivered_at) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(proof.id)
    .bind(proof.order_id)
    .bind(proof.courier_id)
    .bind(&proof.recipient_name)
    .bind(proof.qr_barcode_verified)
    .bind(&proof.digital_signature_base64)
    .bind(&proof.photo_evidence_url)
    .bind(proof.delivered_at)
    .execute(&mut *transaction)
    .await
    .map_err(|error| ApiError::unprocessable(format!("could not persist delivery proof: {error}")))?;
    let now = Utc::now();
    sqlx::query(
        "INSERT INTO billing.delivery_invoices (id, order_id, customer_id, amount_minor, currency, status, created_at, issued_at) VALUES ($1,$2,$3,$4,$5,'issued',$6,$6) ON CONFLICT (order_id) DO NOTHING",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(order.id.0)
    .bind(order.customer_id)
    .bind(order.fare.amount_minor)
    .bind(&order.fare.currency)
    .bind(now)
    .execute(&mut *transaction)
    .await
    .map_err(|error| ApiError::unprocessable(format!("could not issue invoice: {error}")))?;
    sqlx::query(
        "INSERT INTO notifications.notifications (id, recipient_id, channel, title, body, status, created_at) VALUES ($1,$2,'push',$3,$4,'queued',$5)",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(order.customer_id)
    .bind("Teslimat tamamlandı")
    .bind(format!("Siparişiniz {} teslim edildi.", order.id.0))
    .bind(now)
    .execute(&mut *transaction)
    .await
    .map_err(|error| ApiError::unprocessable(format!("could not create notification: {error}")))?;
    sqlx::query(
        "INSERT INTO integrations.event_outbox (id, tenant_id, event_type, aggregate_id, payload) VALUES ($1,$2,'order.delivered',$3,$4)",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(tenant_id)
    .bind(order.id.0)
    .bind(json!({"event_type":"order.delivered","order_id":order.id.0,"status":"delivered","timestamp":order.delivered_at}))
    .execute(&mut *transaction)
    .await
    .map_err(|error| ApiError::unprocessable(format!("could not queue delivery event: {error}")))?;
    transaction.commit().await.map_err(|error| {
        ApiError::unprocessable(format!("could not commit delivery transaction: {error}"))
    })?;
    Ok(order)
}

async fn enqueue_delivery_outbox_event(
    state: &AppState,
    order: &qervon_domain::Order,
    tenant_id: uuid::Uuid,
) -> Result<(), ApiError> {
    let Some(pool) = &state.postgres_pool else {
        return Ok(());
    };
    sqlx::query(
        "INSERT INTO integrations.event_outbox (id, tenant_id, event_type, aggregate_id, payload) \
         VALUES ($1, $2, 'order.delivered', $3, $4)",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(tenant_id)
    .bind(order.id.0)
    .bind(json!({"event_type":"order.delivered","order_id":order.id.0,"status":"delivered","timestamp":order.delivered_at}))
    .execute(pool)
    .await
    .map_err(|error| ApiError::unprocessable(format!("could not queue delivery event: {error}")))?;
    Ok(())
}

async fn create_delivery_financial_records(
    state: &AppState,
    order: &qervon_domain::Order,
) -> Result<(), ApiError> {
    if state
        .billing
        .find_invoice_for_order(order.id)
        .await?
        .is_none()
    {
        let invoice = state
            .billing
            .create_invoice(CreateInvoiceInput {
                order_id: order.id,
                customer_id: order.customer_id,
                amount: order.fare.clone(),
            })
            .await?;
        state.billing.issue_invoice(invoice.id).await?;
    }
    state
        .notifications
        .send(SendNotificationInput {
            recipient_id: order.customer_id,
            channel: NotificationChannel::Push,
            title: "Teslimat tamamlandı".into(),
            body: format!("Siparişiniz {} teslim edildi.", order.id.0),
        })
        .await?;
    Ok(())
}

async fn require_courier_subject(state: &AppState, claims: &AccessClaims) -> Result<(), ApiError> {
    if state.tenants.find_courier_tenant(claims.subject).await? != Some(TenantId(claims.tenant_id))
    {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "courier profile does not belong to this tenant".into(),
        });
    }
    Ok(())
}

async fn require_courier_order(
    state: &AppState,
    order_id: OrderId,
    claims: &AccessClaims,
) -> Result<(), ApiError> {
    require_courier_subject(state, claims).await?;
    if state.tenants.find_order_tenant(order_id).await? != Some(TenantId(claims.tenant_id)) {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "order does not belong to this tenant".into(),
        });
    }
    let order = state.orders.get_order(order_id).await?;
    if order.assigned_courier_id != Some(claims.subject) {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "order is not assigned to this courier".into(),
        });
    }
    Ok(())
}

async fn list_live_locations(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<crate::state::LocationUpdateEvent>>, ApiError> {
    if claims.role == UserRole::Customer {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "customers must use order tracking".into(),
        });
    }
    let locations = state
        .latest_locations
        .read()
        .map_err(|_| ApiError::unprocessable("live location cache is unavailable"))?
        .values()
        .filter(|event| event.tenant_id == claims.tenant_id)
        .cloned()
        .collect();
    Ok(Json(locations))
}

async fn order_tracking(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<crate::state::LocationUpdateEvent>, ApiError> {
    let tenant_id = TenantId(claims.tenant_id);
    if state.tenants.find_order_tenant(OrderId(order_id)).await? != Some(tenant_id) {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "order does not belong to this tenant".into(),
        });
    }
    let order = state.orders.get_order(OrderId(order_id)).await?;
    if claims.role == UserRole::Customer && order.customer_id != claims.subject {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "customers can only track their own order".into(),
        });
    }
    let courier_id = order
        .assigned_courier_id
        .ok_or_else(|| ApiError::unprocessable("order does not have an assigned courier"))?;
    let event = state
        .latest_locations
        .read()
        .map_err(|_| ApiError::unprocessable("live location cache is unavailable"))?
        .get(&courier_id)
        .filter(|event| event.tenant_id == claims.tenant_id)
        .cloned()
        .ok_or_else(|| ApiError::unprocessable("courier location is not available"))?;
    Ok(Json(event))
}

async fn operations_overview(
    State(state): State<AppState>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<OperationsOverviewResponse>, ApiError> {
    let tenant_id = claims
        .as_ref()
        .map(|Extension(claims)| TenantId(claims.tenant_id));
    let mut active_orders = 0;
    let mut pending_orders = 0;
    let mut in_transit_orders = 0;
    let mut revenue_by_currency = BTreeMap::<String, i64>::new();
    for order in state.orders.list_all().await? {
        if tenant_id.is_some() && state.tenants.find_order_tenant(order.id).await? != tenant_id {
            continue;
        }
        match order.status {
            qervon_domain::OrderStatus::Pending => {
                active_orders += 1;
                pending_orders += 1;
            }
            qervon_domain::OrderStatus::CourierAssigned | qervon_domain::OrderStatus::InTransit => {
                active_orders += 1;
                if order.status == qervon_domain::OrderStatus::InTransit {
                    in_transit_orders += 1;
                }
            }
            qervon_domain::OrderStatus::Delivered => {
                *revenue_by_currency
                    .entry(order.fare.currency.clone())
                    .or_default() += order.fare.amount_minor;
            }
            qervon_domain::OrderStatus::Cancelled => {}
        }
    }
    let mut available_couriers = 0;
    let mut busy_couriers = 0;
    let mut offline_couriers = 0;
    for courier in state.couriers.list_all_couriers().await? {
        if tenant_id.is_some() && state.tenants.find_courier_tenant(courier.id).await? != tenant_id
        {
            continue;
        }
        match courier.status {
            qervon_domain::CourierStatus::Available => available_couriers += 1,
            qervon_domain::CourierStatus::Busy => busy_couriers += 1,
            qervon_domain::CourierStatus::Offline => offline_couriers += 1,
        }
    }
    Ok(Json(OperationsOverviewResponse {
        active_orders,
        pending_orders,
        in_transit_orders,
        available_couriers,
        busy_couriers,
        offline_couriers,
        delivered_revenue_by_currency: revenue_by_currency
            .into_iter()
            .map(|(currency, amount_minor)| qervon_api_contracts::MoneyDto {
                amount_minor,
                currency,
            })
            .collect(),
    }))
}

async fn create_customer_order(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<CreateCustomerOrderRequest>,
) -> Result<(StatusCode, Json<OrderResponse>), ApiError> {
    let order = state
        .orders
        .create_order(CreateOrderInput {
            customer_id: claims.subject,
            pickup: to_address(request.pickup)?,
            dropoff: to_address(request.dropoff)?,
            fare: Money::new(request.fare_amount_minor, request.fare_currency)?,
        })
        .await?;
    state
        .tenants
        .bind_order(TenantId(claims.tenant_id), order.id)
        .await?;
    Ok((StatusCode::CREATED, Json((&order).into())))
}

async fn get_customer_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<qervon_domain::CustomerProfile>, ApiError> {
    Ok(Json(
        state
            .customers
            .get_profile_by_user(UserId(claims.subject))
            .await?,
    ))
}

async fn add_customer_address(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<CreateCustomerAddressRequest>,
) -> Result<Json<qervon_domain::CustomerProfile>, ApiError> {
    let profile = state
        .customers
        .get_profile_by_user(UserId(claims.subject))
        .await?;
    let location = Location::new(request.latitude, request.longitude)?;
    Ok(Json(
        state
            .customers
            .add_address(profile.id, request.label, location, request.full_address)
            .await?,
    ))
}

async fn remove_customer_address(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Path(address_id): Path<uuid::Uuid>,
) -> Result<Json<qervon_domain::CustomerProfile>, ApiError> {
    let profile = state
        .customers
        .get_profile_by_user(UserId(claims.subject))
        .await?;
    Ok(Json(
        state
            .customers
            .remove_address(profile.id, address_id)
            .await?,
    ))
}

async fn list_customer_orders(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<OrderResponse>>, ApiError> {
    let mut response = Vec::new();
    for order in state.orders.list_all().await? {
        if order.customer_id == claims.subject
            && state.tenants.find_order_tenant(order.id).await? == Some(TenantId(claims.tenant_id))
        {
            response.push((&order).into());
        }
    }
    Ok(Json(response))
}

async fn get_customer_order_invoice(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<qervon_domain::Invoice>, ApiError> {
    let order = state.orders.get_order(OrderId(order_id)).await?;
    if order.customer_id != claims.subject
        || state.tenants.find_order_tenant(order.id).await? != Some(TenantId(claims.tenant_id))
    {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "order does not belong to this customer".into(),
        });
    }
    state
        .billing
        .find_invoice_for_order(order.id)
        .await?
        .map(Json)
        .ok_or_else(|| ApiError::unprocessable("invoice is not available"))
}

async fn get_customer_order_proof(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<qervon_domain::ProofOfDeliveryRecord>, ApiError> {
    let order = state.orders.get_order(OrderId(order_id)).await?;
    if order.customer_id != claims.subject
        || state.tenants.find_order_tenant(order.id).await? != Some(TenantId(claims.tenant_id))
    {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "order does not belong to this customer".into(),
        });
    }
    state
        .proofs_of_delivery
        .find_by_order(order.id)
        .await?
        .map(Json)
        .ok_or_else(|| ApiError::unprocessable("proof of delivery is not available"))
}

async fn list_customer_notifications(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<qervon_domain::Notification>>, ApiError> {
    Ok(Json(
        state
            .notifications
            .list_for_recipient(claims.subject)
            .await?,
    ))
}

#[derive(Deserialize)]
struct CreateWebhookRequest {
    endpoint_url: String,
    event_types: Vec<String>,
}

#[derive(Deserialize)]
struct PushSubscriptionKeys {
    p256dh: String,
    auth: String,
}

#[derive(Deserialize)]
struct PushSubscriptionRequest {
    endpoint: String,
    keys: PushSubscriptionKeys,
}

async fn get_push_config() -> Result<Json<Value>, ApiError> {
    let public_key = std::env::var("QERVON_WEB_PUSH_VAPID_PUBLIC_KEY").map_err(|_| {
        ApiError::unprocessable("browser push is not configured on this deployment")
    })?;
    Ok(Json(json!({ "vapid_public_key": public_key })))
}

async fn upsert_push_subscription(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<PushSubscriptionRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let endpoint = request.endpoint.trim();
    if !endpoint.starts_with("https://") || endpoint.len() > 2_048 {
        return Err(ApiError::unprocessable(
            "push endpoint must be a bounded HTTPS URL",
        ));
    }
    if request.keys.p256dh.trim().is_empty()
        || request.keys.auth.trim().is_empty()
        || request.keys.p256dh.len() > 512
        || request.keys.auth.len() > 256
    {
        return Err(ApiError::unprocessable(
            "push subscription keys are invalid",
        ));
    }
    let pool = state
        .postgres_pool
        .as_ref()
        .ok_or_else(|| ApiError::unprocessable("push subscriptions require PostgreSQL storage"))?;
    let id = sqlx::query_scalar::<_, uuid::Uuid>(
        "INSERT INTO notifications.web_push_subscriptions \
         (id, user_id, endpoint, p256dh, auth) VALUES ($1,$2,$3,$4,$5) \
         ON CONFLICT (endpoint) DO UPDATE \
         SET p256dh = EXCLUDED.p256dh, auth = EXCLUDED.auth, updated_at = now() \
         WHERE notifications.web_push_subscriptions.user_id = EXCLUDED.user_id \
         RETURNING id",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(claims.subject)
    .bind(endpoint)
    .bind(request.keys.p256dh.trim())
    .bind(request.keys.auth.trim())
    .fetch_optional(pool)
    .await
    .map_err(|error| ApiError::unprocessable(format!("could not save push subscription: {error}")))?
    .ok_or_else(|| ApiError::unprocessable("push endpoint belongs to another user"))?;
    Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
}

async fn delete_push_subscription(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<PushSubscriptionRequest>,
) -> Result<StatusCode, ApiError> {
    let pool = state
        .postgres_pool
        .as_ref()
        .ok_or_else(|| ApiError::unprocessable("push subscriptions require PostgreSQL storage"))?;
    sqlx::query(
        "DELETE FROM notifications.web_push_subscriptions WHERE endpoint = $1 AND user_id = $2",
    )
    .bind(request.endpoint.trim())
    .bind(claims.subject)
    .execute(pool)
    .await
    .map_err(|error| {
        ApiError::unprocessable(format!("could not remove push subscription: {error}"))
    })?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_customer_webhook(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<CreateWebhookRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let signing_secret = format!("qvwh_{}", uuid::Uuid::now_v7());
    let secret_hash = format!("{:x}", Sha256::digest(signing_secret.as_bytes()));
    let subscription = qervon_domain::WebhookSubscription::create(
        TenantId(claims.tenant_id),
        request.endpoint_url,
        request.event_types,
        secret_hash,
    )?;
    state.webhooks.create(&subscription).await?;
    if let Some(pool) = &state.postgres_pool {
        let encrypted_secret = encrypt_webhook_secret(&signing_secret)?;
        sqlx::query("UPDATE integrations.webhooks SET encrypted_secret = $2 WHERE id = $1")
            .bind(subscription.id)
            .bind(encrypted_secret)
            .execute(pool)
            .await
            .map_err(|error| {
                ApiError::unprocessable(format!("could not secure webhook secret: {error}"))
            })?;
    }
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "id": subscription.id,
            "endpoint_url": subscription.endpoint_url,
            "event_types": subscription.event_types,
            "enabled": subscription.enabled,
            "created_at": subscription.created_at,
            "signing_secret": signing_secret,
        })),
    ))
}

fn encrypt_webhook_secret(secret: &str) -> Result<Vec<u8>, ApiError> {
    let encoded_key = std::env::var("QERVON_WEBHOOK_ENCRYPTION_KEY").map_err(|_| {
        ApiError::unprocessable("QERVON_WEBHOOK_ENCRYPTION_KEY is required for PostgreSQL webhooks")
    })?;
    let key = BASE64.decode(encoded_key).map_err(|_| {
        ApiError::unprocessable("QERVON_WEBHOOK_ENCRYPTION_KEY must be base64 encoded")
    })?;
    if key.len() != 32 {
        return Err(ApiError::unprocessable(
            "QERVON_WEBHOOK_ENCRYPTION_KEY must decode to 32 bytes",
        ));
    }
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|_| ApiError::unprocessable("invalid webhook encryption key"))?;
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let mut encrypted = nonce.to_vec();
    encrypted.extend(
        cipher
            .encrypt(&nonce, secret.as_bytes())
            .map_err(|_| ApiError::unprocessable("could not encrypt webhook signing secret"))?,
    );
    Ok(encrypted)
}

async fn list_customer_webhooks(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<Value>>, ApiError> {
    let subscriptions = state
        .webhooks
        .list_for_tenant(TenantId(claims.tenant_id))
        .await?;
    Ok(Json(
        subscriptions
            .into_iter()
            .map(|subscription| {
                json!({
                    "id": subscription.id,
                    "endpoint_url": subscription.endpoint_url,
                    "event_types": subscription.event_types,
                    "enabled": subscription.enabled,
                    "created_at": subscription.created_at,
                })
            })
            .collect(),
    ))
}

async fn delete_customer_webhook(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .webhooks
        .delete(TenantId(claims.tenant_id), id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn create_order(
    State(state): State<AppState>,
    claims: Option<Extension<AccessClaims>>,
    Json(request): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<OrderResponse>), ApiError> {
    let order = state
        .orders
        .create_order(CreateOrderInput {
            customer_id: request.customer_id,
            pickup: to_address(request.pickup)?,
            dropoff: to_address(request.dropoff)?,
            fare: Money::new(request.fare_amount_minor, request.fare_currency)?,
        })
        .await?;
    if let Some(Extension(claims)) = claims {
        state
            .tenants
            .bind_order(TenantId(claims.tenant_id), order.id)
            .await?;
    }
    Ok((StatusCode::CREATED, Json((&order).into())))
}

async fn get_order(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<OrderResponse>, ApiError> {
    require_order_tenant(&state, OrderId(order_id), claims.as_ref()).await?;
    let order = state.orders.get_order(OrderId(order_id)).await?;
    Ok(Json((&order).into()))
}

async fn require_order_tenant(
    state: &AppState,
    order_id: OrderId,
    claims: Option<&Extension<AccessClaims>>,
) -> Result<(), ApiError> {
    let Some(Extension(claims)) = claims else {
        return Ok(());
    };
    if state.tenants.find_order_tenant(order_id).await? != Some(TenantId(claims.tenant_id)) {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "order does not belong to this tenant".into(),
        });
    }
    Ok(())
}

async fn assign_courier(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    claims: Option<Extension<AccessClaims>>,
    Json(request): Json<AssignCourierRequest>,
) -> Result<(StatusCode, Json<qervon_api_contracts::AssignmentResponse>), ApiError> {
    let order_id = OrderId(order_id);
    require_order_tenant(&state, order_id, claims.as_ref()).await?;
    let assignment = match request.courier_id {
        Some(courier_id) => {
            if let Some(Extension(claims)) = &claims {
                if state.tenants.find_courier_tenant(courier_id).await?
                    != Some(TenantId(claims.tenant_id))
                {
                    return Err(ApiError {
                        status: StatusCode::FORBIDDEN,
                        detail: "courier does not belong to this tenant".into(),
                    });
                }
            }
            state.dispatch.assign_courier(order_id, courier_id).await?
        }
        None => {
            if let Some(Extension(claims)) = &claims {
                let tenant_id = TenantId(claims.tenant_id);
                let mut tenant_candidates = Vec::new();
                for courier in state.couriers.list_available_couriers().await? {
                    if state.tenants.find_courier_tenant(courier.id).await? == Some(tenant_id) {
                        tenant_candidates.push(courier);
                    }
                }
                state
                    .dispatch
                    .auto_assign_from_candidates(order_id, &tenant_candidates)
                    .await?
            } else {
                state.dispatch.auto_assign(order_id).await?
            }
        }
    };
    Ok((StatusCode::OK, Json((&assignment).into())))
}

async fn start_transit(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<OrderResponse>, ApiError> {
    let order_id = OrderId(order_id);
    require_order_tenant(&state, order_id, claims.as_ref()).await?;
    let order = state.dispatch.start_transit(order_id).await?;
    Ok(Json((&order).into()))
}

async fn deliver_order(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<OrderResponse>, ApiError> {
    let order_id = OrderId(order_id);
    require_order_tenant(&state, order_id, claims.as_ref()).await?;
    let order = state.dispatch.deliver_order(order_id).await?;
    create_delivery_financial_records(&state, &order).await?;
    Ok(Json((&order).into()))
}

async fn cancel_order(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<OrderResponse>, ApiError> {
    let order_id = OrderId(order_id);
    require_order_tenant(&state, order_id, claims.as_ref()).await?;
    let order = state.dispatch.cancel_order(order_id).await?;
    Ok(Json((&order).into()))
}

async fn list_orders(
    State(state): State<AppState>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<Vec<OrderResponse>>, ApiError> {
    let orders = state.orders.list_all().await?;
    let mut response = Vec::new();
    for order in orders {
        if let Some(Extension(claims)) = &claims {
            if state.tenants.find_order_tenant(order.id).await? != Some(TenantId(claims.tenant_id))
            {
                continue;
            }
        }
        response.push((&order).into());
    }
    Ok(Json(response))
}

fn to_address(dto: AddressDto) -> Result<Address, ApiError> {
    Ok(Address {
        location: Location::new(dto.latitude, dto.longitude)?,
        label: dto.label,
    })
}
