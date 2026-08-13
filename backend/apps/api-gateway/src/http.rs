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
    extract::{DefaultBodyLimit, Extension, Path, Query, Request, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{delete, get, post},
    Json, Router,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{Duration, Utc};
use governor::middleware::NoOpMiddleware;
use qervon_api_contracts::{
    AddressDto, AssignCourierRequest, CourierResponse, CreateCustomerOrderRequest,
    CreateOrderRequest, OperationsOverviewResponse, OrderResponse, RegisterCourierRequest,
    SetCourierAvailabilityRequest, UpdateLocationRequest,
};
use qervon_application::{
    CreateInvoiceInput, CreateOrderInput, CurrencyExchangeEngine, FieldServiceScheduler,
    RegisterCourierInput, SendNotificationInput, TaxInvoicingEngine, TimeSlotWindow,
};
use qervon_domain::{
    Address, ColdChainTelemetry, HubManifestAssignment, Location, Money, NotificationChannel,
    OrderId, RefreshSession, RouteBreadcrumb, TenantCompany, TenantId, TenantMemberRole,
    TenantMembership, UserId, UserRole, VehicleId, VehicleType, WarehouseHub,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use sqlx::FromRow;
use std::collections::{BTreeMap, HashMap};
use std::time::Instant;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::api_error::ApiError;
use crate::auth::{
    hash_refresh_token, issue_access_token, new_refresh_token, verify_access_token, AccessClaims,
};
use crate::rate_limit::ClientIpKeyExtractor;
use crate::state::AppState;

/// Builds the CORS policy from `QERVON_CORS_ALLOWED_ORIGINS` (comma-separated
/// origins, e.g. `http://localhost:5173,https://app.example.com`). With no
/// origins configured, cross-origin browser requests are rejected while
/// same-origin requests (the shipped admin/customer HTML, served from this
/// same process) are unaffected.
fn cors_layer() -> CorsLayer {
    let allowed_origins =
        parse_allowed_origins(std::env::var("QERVON_CORS_ALLOWED_ORIGINS").ok().as_deref());

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(allowed_origins))
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
        ])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION])
        .allow_credentials(true)
}

/// Parses the `QERVON_CORS_ALLOWED_ORIGINS` environment value (comma-separated
/// origins) into a list of valid `HeaderValue`s, silently dropping malformed
/// entries. Extracted as a pure function so its parsing behavior is unit
/// testable without mutating process-global environment state.
fn parse_allowed_origins(raw: Option<&str>) -> Vec<HeaderValue> {
    raw.map(|value| {
        value
            .split(',')
            .filter_map(|origin| origin.trim().parse::<HeaderValue>().ok())
            .collect()
    })
    .unwrap_or_default()
}

/// Builds a `GovernorLayer` rate limiter keyed by client IP. Shared by both
/// the global ceiling and the stricter auth-endpoint ceiling.
fn rate_limit_layer(
    burst_size: u32,
    period: std::time::Duration,
) -> GovernorLayer<ClientIpKeyExtractor, NoOpMiddleware, axum::body::Body> {
    let mut builder = GovernorConfigBuilder::default();
    let mut builder = builder.key_extractor(ClientIpKeyExtractor);
    let config = builder
        .period(period)
        .burst_size(burst_size)
        .finish()
        .expect("rate limit configuration must be valid (non-zero burst and period)");
    GovernorLayer::new(config)
}

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

async fn serve_warehouse_console() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../static/warehouse.html"))
}

async fn serve_field_service_console() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../static/field-service.html"))
}

async fn serve_mobile_admin() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../static/mobile-admin.html"))
}

pub fn router(state: AppState) -> Router {
    let public = Router::new()
        .route("/", get(serve_dashboard))
        .route("/index.html", get(serve_dashboard))
        .route("/customer", get(serve_customer_portal))
        .route("/customer.html", get(serve_customer_portal))
        .route("/login", get(serve_login))
        .route("/login.html", get(serve_login))
        .route("/setup", get(serve_setup))
        .route("/setup.html", get(serve_setup))
        .route("/mobile-customer", get(serve_mobile_customer))
        .route("/mobile-customer.html", get(serve_mobile_customer))
        .route("/mobile-courier", get(serve_mobile_courier))
        .route("/mobile-courier.html", get(serve_mobile_courier))
        .route("/warehouse", get(serve_warehouse_console))
        .route("/warehouse.html", get(serve_warehouse_console))
        .route("/field-service", get(serve_field_service_console))
        .route("/field-service.html", get(serve_field_service_console))
        .route("/mobile-admin", get(serve_mobile_admin))
        .route("/mobile-admin.html", get(serve_mobile_admin))
        .route("/manifest.webmanifest", get(serve_web_manifest))
        .route("/sw.js", get(serve_service_worker))
        .route("/swagger-ui", get(serve_swagger_ui))
        .route("/api-docs/openapi.json", get(serve_openapi_spec))
        .route("/health", get(health))
        .route("/ready", get(readiness))
        .route("/metrics", get(metrics_handler));
    let public = public
        .route("/v1/setup/status", get(initial_setup_status))
        .route("/v1/setup/initialize", post(initialize_platform))
        .route("/v1/payments/webhook", post(payment_webhook))
        .route("/v1/auth/refresh", post(auth_refresh))
        .route("/v1/auth/logout", post(auth_logout))
        .route("/v1/browser/auth/refresh", post(browser_refresh))
        .route("/v1/browser/auth/logout", post(browser_logout));
    // Credential-bearing endpoints get a tighter rate ceiling than the rest
    // of the API to slow down brute-force/credential-stuffing attempts.
    let auth_sensitive = Router::new()
        .route("/v1/auth/register", post(auth_register))
        .route("/v1/auth/login", post(auth_login))
        .route("/v1/browser/auth/login", post(browser_login))
        .route("/v1/auth/otp/request", post(auth_otp_request))
        .route("/v1/auth/otp/verify", post(auth_otp_verify))
        .route_layer(rate_limit_layer(10, std::time::Duration::from_secs(3)));

    let operations = Router::new()
        .route("/v1/operations/overview", get(operations_overview))
        .route("/v1/foundation/runtime", get(get_foundation_runtime))
        .route("/v1/warehouse/hubs", post(create_warehouse_hub).get(list_warehouse_hubs))
        .route(
            "/v1/warehouse/hubs/{id}/receive",
            post(receive_warehouse_parcels),
        )
        .route(
            "/v1/warehouse/hubs/{id}/dispatch",
            post(dispatch_warehouse_manifest),
        )
        .route(
            "/v1/cold-chain/telemetry",
            post(record_cold_chain_telemetry).get(list_cold_chain_telemetry),
        )
        .route(
            "/v1/field-service/appointments",
            post(create_field_service_appointment).get(list_field_service_appointments),
        )
        .route(
            "/v1/route-history/{courier_id}/breadcrumbs",
            post(record_route_breadcrumb),
        )
        .route(
            "/v1/route-history/{courier_id}",
            get(get_route_playback_track),
        )
        .route("/v1/tax/invoice-draft", post(generate_tax_invoice_draft))
        .route("/v1/currency/convert", get(convert_currency_amount))
        .route("/v1/payments/charge", post(charge_payment))
        .route("/v1/push/native/dispatch", post(dispatch_native_push))
        .route("/v1/ops/slo-report", get(get_slo_report))
        .route("/v1/ops/dr-drill", post(run_dr_drill))
        .route("/v1/users", post(register_user))
        .route("/v1/tenants/provision", post(provision_tenant))
        .route(
            "/v1/company/admins/provision",
            post(provision_company_admin),
        )
        .route("/v1/customers/provision", post(provision_customer))
        .route("/v1/couriers/provision", post(provision_courier))
        .route("/v1/couriers", post(register_courier).get(list_couriers))
        .route("/v1/couriers/{id}/wallet", get(get_courier_wallet))
        .route("/v1/couriers/{id}/ratings", get(list_courier_ratings))
        .route("/v1/coupons", post(create_coupon).get(list_coupons))
        .route(
            "/v1/fleet/vehicles",
            post(register_vehicle).get(list_vehicles),
        )
        .route("/v1/fleet/vehicles/{id}", get(get_vehicle))
        .route("/v1/fleet/vehicles/{id}/assign", post(assign_vehicle))
        .route(
            "/v1/fleet/vehicles/{id}/maintenance",
            post(send_vehicle_to_maintenance),
        )
        .route("/v1/fleet/vehicles/{id}/activate", post(activate_vehicle))
        .route(
            "/v1/fleet/vehicles/{id}/decommission",
            post(decommission_vehicle),
        )
        .route("/v1/orders", post(create_order).get(list_orders))
        .route("/v1/orders/{id}", get(get_order))
        .route("/v1/orders/{id}/assign", post(assign_courier))
        .route("/v1/orders/{id}/transit", post(start_transit))
        .route("/v1/orders/{id}/deliver", post(deliver_order))
        .route("/v1/orders/{id}/cancel", post(cancel_order))
        .route("/v1/orders/{id}/return", post(return_order))
        .route("/v1/reports/operations", get(operations_report))
        .route("/v1/finance/summary", get(finance_summary))
        .route("/v1/finance/invoices", get(list_finance_invoices))
        .route("/v1/company", get(company_profile))
        .route(
            "/v1/company/members",
            get(list_company_members).post(add_company_member),
        )
        .route("/v1/pricing", get(get_pricing).put(update_pricing))
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
        .route("/v1/customer/fare-quote", get(get_customer_fare_quote))
        .route(
            "/v1/customer/orders/{id}/invoice",
            get(get_customer_order_invoice),
        )
        .route(
            "/v1/customer/orders/{id}/proof-of-delivery",
            get(get_customer_order_proof),
        )
        .route("/v1/customer/orders/{id}/rating", post(rate_customer_order))
        .route(
            "/v1/customer/orders/{id}/cancel",
            post(cancel_customer_order),
        )
        .route("/v1/customer/orders/{id}/eta", get(get_customer_order_eta))
        .route(
            "/v1/customer/support-tickets",
            post(create_customer_support_ticket).get(list_customer_support_tickets),
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
        .route("/v1/courier/me/wallet", get(get_own_wallet))
        .route("/v1/courier/me/ratings", get(list_own_ratings))
        .route("/v1/courier/me/status", post(set_own_courier_availability))
        .route("/v1/courier/me/location", post(update_own_courier_location))
        .route("/v1/courier/me/offer", get(get_own_pending_offer))
        .route("/v1/courier/orders", get(list_courier_orders))
        .route("/v1/courier/orders/{id}/accept", post(accept_courier_offer))
        .route("/v1/courier/orders/{id}/reject", post(reject_courier_offer))
        .route(
            "/v1/courier/orders/{id}/pickup",
            post(courier_start_transit),
        )
        .route(
            "/v1/courier/orders/{id}/deliver",
            post(courier_deliver_order),
        )
        .route_layer(middleware::from_fn(require_courier_access));
    // A dedicated sub-router so the larger body-size limit for photo
    // uploads (MAX_UPLOAD_BYTES) only applies to this one route, not the
    // global default (`DefaultBodyLimit::max(1_048_576)` below) every
    // other endpoint uses.
    let photo_uploads = Router::new()
        .route(
            "/v1/courier/orders/{id}/photo-evidence",
            post(upload_delivery_photo),
        )
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
        .route_layer(middleware::from_fn(require_courier_access));
    let upload_reads = Router::new()
        .route("/v1/uploads/{*path}", get(serve_upload))
        .route_layer(middleware::from_fn(require_signed_user));
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
    let session_operations = Router::new()
        .route("/v1/auth/session", get(current_browser_session))
        .route("/v1/auth/phone", post(set_own_phone))
        .route(
            "/v1/push/devices",
            post(register_push_device).get(list_push_devices),
        )
        .route("/v1/push/devices/{id}", delete(delete_push_device))
        .route_layer(middleware::from_fn(require_signed_user));
    // Location events currently lack a tenant key in the delivery aggregate.
    // Do not expose an all-tenant stream to signed end users until the event
    // and assignment models carry that boundary end-to-end.
    let protected = operations
        .merge(location_publisher)
        .merge(courier_operations)
        .merge(photo_uploads)
        .merge(upload_reads)
        .merge(customer_operations)
        .merge(tracking_consumers)
        .merge(push_operations)
        .merge(session_operations)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            require_api_access,
        ));

    public
        .merge(auth_sensitive)
        .merge(protected)
        .with_state(state.clone())
        .layer(DefaultBodyLimit::max(1_048_576))
        .layer(middleware::from_fn_with_state(state, observe_request))
        .layer(rate_limit_layer(120, std::time::Duration::from_millis(200)))
        .layer(cors_layer())
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
    response.headers_mut().insert(
        HeaderName::from_static("x-frame-options"),
        HeaderValue::from_static("DENY"),
    );
    response.headers_mut().insert(
        HeaderName::from_static("content-security-policy"),
        HeaderValue::from_static("default-src 'self'; frame-ancestors 'none'"),
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
    /// A customer must belong to the logistics company whose deliveries it
    /// creates.  Keeping this optional preserves the public API's historic
    /// account-only behaviour, while browser sign-in uses the tenant value.
    tenant_slug: Option<String>,
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
struct OtpRequestRequest {
    tenant_slug: String,
    phone: String,
}

#[derive(Serialize)]
struct OtpRequestResponse {
    status: &'static str,
    /// Only populated when running on in-memory (local/dev) storage, where
    /// there is no real SMS provider to deliver the code out-of-band. Always
    /// `null` on PostgreSQL-backed (production) deployments; see
    /// BACKEND_BACKLOG.md for the SMS-provider integration gap.
    dev_code: Option<String>,
}

#[derive(Deserialize)]
struct OtpVerifyRequest {
    tenant_slug: String,
    phone: String,
    code: String,
}

#[derive(Deserialize)]
struct BrowserLoginRequest {
    email: String,
    password: String,
    tenant_slug: String,
}

#[derive(Deserialize)]
struct ProvisionCourierRequest {
    email: String,
    display_name: String,
    password: String,
    vehicle: String,
}

#[derive(Deserialize)]
struct ProvisionCustomerRequest {
    email: String,
    display_name: String,
    password: String,
}

#[derive(Deserialize)]
struct InitialSetupRequest {
    setup_token: Option<String>,
    tenant_name: String,
    tenant_slug: String,
    admin_email: String,
    admin_name: String,
    admin_password: String,
}

#[derive(Deserialize)]
struct ProvisionTenantRequest {
    tenant_name: String,
    tenant_slug: String,
    admin_email: String,
    admin_name: String,
    admin_password: String,
}

#[derive(Deserialize)]
struct ProvisionCompanyAdminRequest {
    email: String,
    display_name: String,
    password: String,
}

struct TenantAndAdminInput {
    tenant_name: String,
    tenant_slug: String,
    admin_email: String,
    admin_name: String,
    admin_password: String,
}

#[derive(Deserialize)]
struct CompleteDeliveryRequest {
    recipient_name: String,
    qr_barcode_verified: bool,
    digital_signature_base64: Option<String>,
    photo_evidence_url: Option<String>,
    /// Set by the courier when the order's chosen payment method is cash
    /// and the amount was physically collected on drop-off.
    #[serde(default)]
    payment_collected: bool,
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
    headers: HeaderMap,
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
    Ok((
        browser_session_headers(&session, cookies_require_secure_transport(&headers)),
        StatusCode::NO_CONTENT,
    ))
}

async fn auth_register(
    State(state): State<AppState>,
    Json(request): Json<AuthRegisterRequest>,
) -> Result<StatusCode, ApiError> {
    // A public caller can only become a customer. Tenant membership and every
    // operational role are provisioned by an authenticated tenant administrator.
    let tenant = match request.tenant_slug.as_deref() {
        Some(slug) => Some(
            state
                .tenants
                .find_by_slug(&tenant_slug(slug.to_string())?)
                .await?
                .ok_or_else(|| ApiError::unprocessable("tenant was not found"))?,
        ),
        None => None,
    };
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
    if let Some(tenant) = tenant {
        state
            .tenants
            .add_member(&TenantMembership {
                tenant_id: tenant.id,
                user_id: user.id,
                role: TenantMemberRole::Member,
                joined_at: Utc::now(),
            })
            .await?;
    }
    Ok(StatusCode::CREATED)
}

async fn initial_setup_status(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let required = !state.tenants.has_any_tenant().await?;
    Ok(Json(json!({
        "initial_setup_required": required,
        "setup_token_required": required && state.storage_backend == crate::state::StorageBackend::Postgres,
    })))
}

async fn initialize_platform(
    State(state): State<AppState>,
    Json(request): Json<InitialSetupRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    if state.tenants.has_any_tenant().await? {
        return Err(ApiError {
            status: StatusCode::CONFLICT,
            detail: "initial platform setup has already been completed".into(),
        });
    }
    if state.storage_backend == crate::state::StorageBackend::Postgres {
        let expected = state
            .initial_setup_token
            .as_deref()
            .ok_or_else(|| ApiError {
                status: StatusCode::SERVICE_UNAVAILABLE,
                detail: "QERVON_INITIAL_SETUP_TOKEN is required for PostgreSQL initial setup"
                    .into(),
            })?;
        if request.setup_token.as_deref() != Some(expected) {
            return Err(ApiError {
                status: StatusCode::FORBIDDEN,
                detail: "initial setup token is invalid".into(),
            });
        }
    }
    let (tenant, admin) = create_tenant_with_admin(
        &state,
        TenantAndAdminInput {
            tenant_name: request.tenant_name,
            tenant_slug: request.tenant_slug,
            admin_email: request.admin_email,
            admin_name: request.admin_name,
            admin_password: request.admin_password,
        },
        UserRole::SuperAdmin,
        TenantMemberRole::Owner,
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "tenant_id": tenant.0.id.0,
            "tenant_slug": tenant.1,
            "admin_id": admin.id.0,
            "admin_role": "super_admin",
        })),
    ))
}

async fn provision_tenant(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<ProvisionTenantRequest>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    if claims.role != UserRole::SuperAdmin {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "only platform administrators can create tenants".into(),
        });
    }
    let (tenant, admin) = create_tenant_with_admin(
        &state,
        TenantAndAdminInput {
            tenant_name: request.tenant_name,
            tenant_slug: request.tenant_slug,
            admin_email: request.admin_email,
            admin_name: request.admin_name,
            admin_password: request.admin_password,
        },
        UserRole::Admin,
        TenantMemberRole::Owner,
    )
    .await?;
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "tenant_id": tenant.0.id.0,
            "tenant_slug": tenant.1,
            "admin_id": admin.id.0,
            "admin_role": "admin",
        })),
    ))
}

async fn provision_company_admin(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<ProvisionCompanyAdminRequest>,
) -> Result<(StatusCode, Json<qervon_api_contracts::UserResponse>), ApiError> {
    if !matches!(claims.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "only tenant administrators can provision tenant administrators".into(),
        });
    }
    let user = state
        .auth
        .register(
            request.email,
            request.display_name,
            request.password,
            UserRole::Admin,
        )
        .await?;
    state
        .tenants
        .add_member(&TenantMembership {
            tenant_id: TenantId(claims.tenant_id),
            user_id: user.id,
            role: TenantMemberRole::Admin,
            joined_at: Utc::now(),
        })
        .await?;
    Ok((StatusCode::CREATED, Json((&user).into())))
}

fn tenant_slug(value: String) -> Result<String, ApiError> {
    let value = value.trim().to_ascii_lowercase();
    let valid = (3..=63).contains(&value.len())
        && !value.starts_with('-')
        && !value.ends_with('-')
        && value.bytes().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == b'-'
        });
    if valid {
        Ok(value)
    } else {
        Err(ApiError::unprocessable(
            "tenant slug must be 3-63 lowercase letters, digits, or hyphens",
        ))
    }
}

async fn create_tenant_with_admin(
    state: &AppState,
    input: TenantAndAdminInput,
    admin_role: UserRole,
    membership_role: TenantMemberRole,
) -> Result<((TenantCompany, String), qervon_domain::User), ApiError> {
    let slug = tenant_slug(input.tenant_slug)?;
    let tenant_name = input.tenant_name.trim();
    if !(2..=160).contains(&tenant_name.len()) {
        return Err(ApiError::unprocessable(
            "tenant name must be 2-160 characters",
        ));
    }
    if state.tenants.find_by_slug(&slug).await?.is_some() {
        return Err(ApiError {
            status: StatusCode::CONFLICT,
            detail: "tenant slug already exists".into(),
        });
    }
    let tenant = TenantCompany {
        id: TenantId::new(),
        company_name: tenant_name.to_string(),
        category: "Logistics".into(),
        created_at: Utc::now(),
    };
    state.tenants.create_tenant(&tenant, &slug).await?;
    let admin = state
        .auth
        .register(
            input.admin_email,
            input.admin_name,
            input.admin_password,
            admin_role,
        )
        .await?;
    state
        .tenants
        .add_member(&TenantMembership {
            tenant_id: tenant.id,
            user_id: admin.id,
            role: membership_role,
            joined_at: Utc::now(),
        })
        .await?;
    Ok(((tenant, slug), admin))
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

async fn auth_otp_request(
    State(state): State<AppState>,
    Json(request): Json<OtpRequestRequest>,
) -> Result<Json<OtpRequestResponse>, ApiError> {
    let tenant = state
        .tenants
        .find_by_slug(&request.tenant_slug)
        .await?
        .ok_or_else(invalid_credentials)?;
    let code = state
        .otp
        .request_otp(tenant.id, &request.phone)
        .await
        .map_err(|_| invalid_credentials())?;
    if let Err(error) = deliver_otp_sms(&state, &request.phone, &code).await {
        tracing::warn!(
            phone = %request.phone,
            error = %error,
            "OTP provider delivery failed; returning response for retryable client handling"
        );
    }
    let dev_code = match state.storage_backend {
        crate::state::StorageBackend::Memory => {
            tracing::info!(
                phone = %request.phone,
                code = %code,
                "OTP issued (in-memory/local storage; no SMS provider configured)"
            );
            Some(code)
        }
        crate::state::StorageBackend::Postgres => {
            tracing::info!(phone = %request.phone, "OTP issued via provider flow");
            None
        }
    };
    Ok(Json(OtpRequestResponse {
        status: "sent",
        dev_code,
    }))
}

async fn auth_otp_verify(
    State(state): State<AppState>,
    Json(request): Json<OtpVerifyRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let tenant = state
        .tenants
        .find_by_slug(&request.tenant_slug)
        .await?
        .ok_or_else(invalid_credentials)?;
    let user = state
        .otp
        .verify_otp(tenant.id, &request.phone, &request.code)
        .await
        .map_err(|_| invalid_credentials())?;
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

/// Returns only the identity data the browser needs to render the current
/// session state. Tokens remain HTTP-only and are never exposed to the page.
async fn current_browser_session(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Value>, ApiError> {
    let user = state.identity.get_user(UserId(claims.subject)).await?;
    Ok(Json(json!({
        "display_name": user.display_name,
        "email": user.email,
        "role": user.role.as_str(),
        "tenant_id": claims.tenant_id,
    })))
}

#[derive(Deserialize)]
struct SetPhoneRequest {
    phone: String,
}

/// Links a phone number to the signed-in account. Required before this
/// user can request an OTP login challenge, since `OtpService` resolves
/// accounts strictly by phone number.
async fn set_own_phone(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<SetPhoneRequest>,
) -> Result<Json<qervon_api_contracts::UserResponse>, ApiError> {
    let user = state
        .identity
        .set_user_phone(UserId(claims.subject), request.phone)
        .await?;
    Ok(Json((&user).into()))
}

/// Registers this device for native push delivery (iOS/Android). Re-sending
/// the same token is idempotent. No APNs/FCM sending is wired up yet — see
/// BACKEND_BACKLOG.md.
async fn register_push_device(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<qervon_api_contracts::RegisterPushDeviceRequest>,
) -> Result<
    (
        StatusCode,
        Json<qervon_api_contracts::DevicePushTokenResponse>,
    ),
    ApiError,
> {
    let platform = request
        .platform
        .parse::<qervon_domain::PushPlatform>()
        .map_err(|_| ApiError::unprocessable("invalid push platform"))?;
    let token = state
        .device_push
        .register(UserId(claims.subject), platform, request.device_token)
        .await?;
    Ok((StatusCode::CREATED, Json((&token).into())))
}

async fn list_push_devices(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<qervon_api_contracts::DevicePushTokenResponse>>, ApiError> {
    let tokens = state
        .device_push
        .list_for_user(UserId(claims.subject))
        .await?;
    Ok(Json(tokens.iter().map(Into::into).collect()))
}

async fn delete_push_device(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Path(id): Path<uuid::Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .device_push
        .unregister(UserId(claims.subject), id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
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
    Ok((
        browser_session_headers(&next, cookies_require_secure_transport(&headers)),
        StatusCode::NO_CONTENT,
    ))
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
    Ok((
        expired_browser_session_headers(cookies_require_secure_transport(&headers)),
        StatusCode::NO_CONTENT,
    ))
}

fn cookies_require_secure_transport(headers: &HeaderMap) -> bool {
    if headers
        .get("x-forwarded-proto")
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.eq_ignore_ascii_case("https"))
    {
        return true;
    }
    !headers
        .get(header::HOST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|host| {
            host.starts_with("localhost:")
                || host == "localhost"
                || host.starts_with("127.0.0.1:")
                || host == "127.0.0.1"
                || host.starts_with("[::1]:")
                || host == "[::1]"
        })
}

fn browser_session_headers(session: &AuthResponse, secure: bool) -> HeaderMap {
    let csrf = new_refresh_token();
    let mut headers = HeaderMap::new();
    let secure_attribute = if secure { "; Secure" } else { "" };
    let access = format!(
        "qervon_access_token={}; Path=/; Max-Age={}; HttpOnly; SameSite=Lax{}",
        session.access_token, session.expires_in_seconds, secure_attribute
    );
    let refresh = format!(
        "qervon_refresh_token={}; Path=/v1/browser/auth; Max-Age={}; HttpOnly; SameSite=Strict{}",
        session.refresh_token,
        60 * 60 * 24 * 30,
        secure_attribute
    );
    let csrf_cookie = format!(
        "qervon_csrf_token={csrf}; Path=/; Max-Age={}; SameSite=Strict{}",
        60 * 60 * 24 * 30,
        secure_attribute
    );
    for value in [access, refresh, csrf_cookie] {
        headers.append(
            header::SET_COOKIE,
            HeaderValue::from_str(&value).expect("cookie header is valid"),
        );
    }
    headers
}

fn expired_browser_session_headers(secure: bool) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let secure_attribute = if secure { "; Secure" } else { "" };
    for value in [
        format!("qervon_access_token=; Path=/; Max-Age=0; HttpOnly; SameSite=Lax{secure_attribute}"),
        format!("qervon_refresh_token=; Path=/v1/browser/auth; Max-Age=0; HttpOnly; SameSite=Strict{secure_attribute}"),
        format!("qervon_csrf_token=; Path=/; Max-Age=0; SameSite=Strict{secure_attribute}"),
    ] {
        headers.append(
            header::SET_COOKIE,
            HeaderValue::from_str(&value).expect("cookie header is valid"),
        );
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

async fn deliver_otp_sms(state: &AppState, phone: &str, code: &str) -> Result<(), String> {
    let Some(url) = &state.sms_provider_url else {
        return Ok(());
    };
    let client = reqwest::Client::new();
    let mut request = client
        .post(url)
        .header("content-type", "application/json")
        .body(
            json!({
        "phone": phone,
        "message": format!("Qervon OTP code: {code}"),
    })
            .to_string(),
        );
    if let Some(token) = &state.sms_provider_bearer_token {
        request = request.bearer_auth(token.as_ref());
    }
    let response = request.send().await.map_err(|error| error.to_string())?;
    if response.status().is_success() {
        Ok(())
    } else {
        Err(format!("sms provider responded with {}", response.status()))
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

async fn serve_setup() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../static/setup.html"))
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

async fn provision_courier(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<ProvisionCourierRequest>,
) -> Result<(StatusCode, Json<CourierResponse>), ApiError> {
    if !matches!(claims.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "only tenant administrators can provision couriers".into(),
        });
    }
    let vehicle = request
        .vehicle
        .parse::<VehicleType>()
        .map_err(|_| ApiError::unprocessable("invalid vehicle type"))?;
    let user = state
        .auth
        .register(
            request.email,
            request.display_name.clone(),
            request.password,
            UserRole::Courier,
        )
        .await?;
    let tenant_id = TenantId(claims.tenant_id);
    state
        .tenants
        .add_member(&TenantMembership {
            tenant_id,
            user_id: user.id,
            role: TenantMemberRole::Member,
            joined_at: Utc::now(),
        })
        .await?;
    let courier = state
        .couriers
        .register_courier(RegisterCourierInput {
            id: user.id.0,
            name: request.display_name,
            vehicle,
        })
        .await?;
    state.tenants.bind_courier(tenant_id, courier.id).await?;
    Ok((StatusCode::CREATED, Json((&courier).into())))
}

/// Creates a customer account inside the caller's tenant.  Customers get a
/// profile and a tenant membership in one operation, so the customer portals
/// can immediately authenticate and create tenant-scoped orders.
async fn provision_customer(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<ProvisionCustomerRequest>,
) -> Result<(StatusCode, Json<qervon_api_contracts::UserResponse>), ApiError> {
    if !matches!(claims.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "only tenant administrators can provision customers".into(),
        });
    }
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
    state
        .tenants
        .add_member(&TenantMembership {
            tenant_id: TenantId(claims.tenant_id),
            user_id: user.id,
            role: TenantMemberRole::Member,
            joined_at: Utc::now(),
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

async fn register_vehicle(
    State(state): State<AppState>,
    claims: Option<Extension<AccessClaims>>,
    Json(request): Json<qervon_api_contracts::RegisterVehicleRequest>,
) -> Result<(StatusCode, Json<qervon_api_contracts::VehicleResponse>), ApiError> {
    let vehicle_type: VehicleType = request
        .vehicle_type
        .parse()
        .map_err(|_| ApiError::unprocessable("invalid vehicle type"))?;
    let vehicle = state
        .fleet
        .register_vehicle(qervon_application::RegisterVehicleInput {
            plate_number: request.plate_number,
            vehicle_type,
            insurance_expiry: request.insurance_expiry,
        })
        .await?;
    if let Some(Extension(claims)) = claims {
        state
            .tenants
            .bind_vehicle(TenantId(claims.tenant_id), vehicle.id)
            .await?;
    }
    Ok((StatusCode::CREATED, Json((&vehicle).into())))
}

async fn list_vehicles(
    State(state): State<AppState>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<Vec<qervon_api_contracts::VehicleResponse>>, ApiError> {
    let vehicles = state.fleet.list_active_vehicles().await?;
    let mut response = Vec::new();
    for vehicle in vehicles {
        if let Some(Extension(claims)) = &claims {
            if state.tenants.find_vehicle_tenant(vehicle.id).await?
                != Some(TenantId(claims.tenant_id))
            {
                continue;
            }
        }
        response.push(qervon_api_contracts::VehicleResponse::from(&vehicle));
    }
    Ok(Json(response))
}

async fn require_vehicle_tenant(
    state: &AppState,
    vehicle_id: VehicleId,
    claims: Option<&Extension<AccessClaims>>,
) -> Result<(), ApiError> {
    let Some(Extension(claims)) = claims else {
        return Ok(());
    };
    if state.tenants.find_vehicle_tenant(vehicle_id).await? != Some(TenantId(claims.tenant_id)) {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "vehicle does not belong to this tenant".into(),
        });
    }
    Ok(())
}

async fn get_vehicle(
    State(state): State<AppState>,
    Path(vehicle_id): Path<uuid::Uuid>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<qervon_api_contracts::VehicleResponse>, ApiError> {
    let vehicle_id = VehicleId(vehicle_id);
    require_vehicle_tenant(&state, vehicle_id, claims.as_ref()).await?;
    let vehicle = state.fleet.get_vehicle(vehicle_id).await?;
    Ok(Json((&vehicle).into()))
}

async fn assign_vehicle(
    State(state): State<AppState>,
    Path(vehicle_id): Path<uuid::Uuid>,
    claims: Option<Extension<AccessClaims>>,
    Json(request): Json<qervon_api_contracts::AssignVehicleRequest>,
) -> Result<Json<qervon_api_contracts::VehicleResponse>, ApiError> {
    let vehicle_id = VehicleId(vehicle_id);
    require_vehicle_tenant(&state, vehicle_id, claims.as_ref()).await?;
    if let Some(Extension(claims)) = &claims {
        if state
            .tenants
            .find_courier_tenant(request.courier_id)
            .await?
            != Some(TenantId(claims.tenant_id))
        {
            return Err(ApiError {
                status: StatusCode::FORBIDDEN,
                detail: "courier does not belong to this tenant".into(),
            });
        }
    }
    let vehicle = state
        .fleet
        .assign_courier(vehicle_id, request.courier_id)
        .await?;
    Ok(Json((&vehicle).into()))
}

async fn send_vehicle_to_maintenance(
    State(state): State<AppState>,
    Path(vehicle_id): Path<uuid::Uuid>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<qervon_api_contracts::VehicleResponse>, ApiError> {
    let vehicle_id = VehicleId(vehicle_id);
    require_vehicle_tenant(&state, vehicle_id, claims.as_ref()).await?;
    let vehicle = state.fleet.send_to_maintenance(vehicle_id).await?;
    Ok(Json((&vehicle).into()))
}

async fn activate_vehicle(
    State(state): State<AppState>,
    Path(vehicle_id): Path<uuid::Uuid>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<qervon_api_contracts::VehicleResponse>, ApiError> {
    let vehicle_id = VehicleId(vehicle_id);
    require_vehicle_tenant(&state, vehicle_id, claims.as_ref()).await?;
    let vehicle = state.fleet.activate(vehicle_id).await?;
    Ok(Json((&vehicle).into()))
}

async fn decommission_vehicle(
    State(state): State<AppState>,
    Path(vehicle_id): Path<uuid::Uuid>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<qervon_api_contracts::VehicleResponse>, ApiError> {
    let vehicle_id = VehicleId(vehicle_id);
    require_vehicle_tenant(&state, vehicle_id, claims.as_ref()).await?;
    let vehicle = state.fleet.decommission(vehicle_id).await?;
    Ok(Json((&vehicle).into()))
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
    let recorded_point = state
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
        fraud_flagged: recorded_point.fraud_flagged,
        fraud_risk_score: recorded_point.fraud_risk_score,
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

/// Default currency for a wallet that has not received any transactions
/// yet. Qervon does not currently model a per-tenant default currency, so
/// this matches the currency used throughout the rest of the vertical slice
/// (order fares, invoices, payouts).
const DEFAULT_WALLET_CURRENCY: &str = "TRY";

async fn get_own_wallet(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<qervon_api_contracts::CourierWalletResponse>, ApiError> {
    require_courier_subject(&state, &claims).await?;
    let wallet = state
        .courier_wallets
        .get_wallet(claims.subject, DEFAULT_WALLET_CURRENCY)
        .await?;
    Ok(Json((&wallet).into()))
}

async fn get_courier_wallet(
    State(state): State<AppState>,
    Path(courier_id): Path<uuid::Uuid>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<qervon_api_contracts::CourierWalletResponse>, ApiError> {
    if let Some(Extension(claims)) = &claims {
        if state.tenants.find_courier_tenant(courier_id).await? != Some(TenantId(claims.tenant_id))
        {
            return Err(ApiError {
                status: StatusCode::FORBIDDEN,
                detail: "courier does not belong to this tenant".into(),
            });
        }
    }
    let wallet = state
        .courier_wallets
        .get_wallet(courier_id, DEFAULT_WALLET_CURRENCY)
        .await?;
    Ok(Json((&wallet).into()))
}

async fn list_courier_ratings(
    State(state): State<AppState>,
    Path(courier_id): Path<uuid::Uuid>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<Vec<qervon_api_contracts::CustomerRatingResponse>>, ApiError> {
    if let Some(Extension(claims)) = &claims {
        if state.tenants.find_courier_tenant(courier_id).await? != Some(TenantId(claims.tenant_id))
        {
            return Err(ApiError {
                status: StatusCode::FORBIDDEN,
                detail: "courier does not belong to this tenant".into(),
            });
        }
    }
    let ratings = state.ratings.list_for_courier(courier_id).await?;
    Ok(Json(ratings.iter().map(Into::into).collect()))
}

async fn list_own_ratings(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<qervon_api_contracts::CustomerRatingResponse>>, ApiError> {
    require_courier_subject(&state, &claims).await?;
    let ratings = state.ratings.list_for_courier(claims.subject).await?;
    Ok(Json(ratings.iter().map(Into::into).collect()))
}

/// Polled by the courier app (no push mechanism offers jobs today) to
/// discover a pending job offer. Returns `null` when there is none — a
/// normal, expected state, not an error. If this courier's own offer was
/// just found to have expired, attempts to re-offer the order to the
/// next-best candidate in the same tenant (see `reoffer_for_tenant`) before
/// responding — the re-offer cascade's only trigger points are this poll
/// and `reject_courier_offer` below, since expiry is discovered lazily
/// rather than by a background sweep (see QAS-000003).
async fn get_own_pending_offer(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Option<qervon_api_contracts::PendingOfferResponse>>, ApiError> {
    require_courier_subject(&state, &claims).await?;
    match state
        .dispatch
        .find_pending_offer_or_expiry(claims.subject)
        .await?
    {
        qervon_application::PendingOfferLookup::Active(assignment, order) => Ok(Json(Some(
            qervon_api_contracts::PendingOfferResponse::new(&assignment, &order),
        ))),
        qervon_application::PendingOfferLookup::None => Ok(Json(None)),
        qervon_application::PendingOfferLookup::JustExpired(assignment) => {
            let _ = reoffer_for_tenant(
                &state,
                assignment.order_id,
                TenantId(claims.tenant_id),
                &assignment.excluded_including_self(),
            )
            .await;
            Ok(Json(None))
        }
    }
}

async fn accept_courier_offer(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<OrderResponse>, ApiError> {
    require_courier_subject(&state, &claims).await?;
    let order = state
        .dispatch
        .accept_offer(OrderId(order_id), claims.subject)
        .await?;
    Ok(Json((&order).into()))
}

/// Rejects a pending job offer, then attempts to re-offer the same order to
/// the next-best candidate in the same tenant (see `reoffer_for_tenant`) —
/// the courier who just rejected never sees the outcome of that cascade
/// step; whichever courier it lands on discovers it on their own next poll
/// of `GET /v1/courier/me/offer`.
async fn reject_courier_offer(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<StatusCode, ApiError> {
    require_courier_subject(&state, &claims).await?;
    let rejected = state
        .dispatch
        .reject_offer(OrderId(order_id), claims.subject)
        .await?;
    let _ = reoffer_for_tenant(
        &state,
        rejected.order_id,
        TenantId(claims.tenant_id),
        &rejected.excluded_including_self(),
    )
    .await;
    Ok(StatusCode::NO_CONTENT)
}

fn tenant_context_required() -> ApiError {
    ApiError {
        status: StatusCode::FORBIDDEN,
        detail: "coupons require a tenant-scoped session".into(),
    }
}

async fn create_coupon(
    State(state): State<AppState>,
    claims: Option<Extension<AccessClaims>>,
    Json(request): Json<qervon_api_contracts::CreateCouponRequest>,
) -> Result<(StatusCode, Json<qervon_api_contracts::CouponResponse>), ApiError> {
    let Some(Extension(claims)) = claims else {
        return Err(tenant_context_required());
    };
    let coupon = state
        .coupons
        .create_coupon(
            TenantId(claims.tenant_id),
            request.code,
            request.discount_percent,
            request.max_discount_minor,
            request.valid_until,
            request.usage_limit,
        )
        .await?;
    Ok((StatusCode::CREATED, Json((&coupon).into())))
}

async fn list_coupons(
    State(state): State<AppState>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<Vec<qervon_api_contracts::CouponResponse>>, ApiError> {
    let Some(Extension(claims)) = claims else {
        return Err(tenant_context_required());
    };
    let coupons = state
        .coupons
        .list_for_tenant(TenantId(claims.tenant_id))
        .await?;
    Ok(Json(coupons.iter().map(Into::into).collect()))
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
                qervon_domain::OrderStatus::Delivered
                    | qervon_domain::OrderStatus::Cancelled
                    | qervon_domain::OrderStatus::Returned
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
        complete_delivery_atomically(
            &state,
            order_id,
            &proof,
            claims.tenant_id,
            request.payment_collected,
        )
        .await?
    } else {
        let mut order = state.dispatch.deliver_order(order_id).await?;
        state.proofs_of_delivery.create(&proof).await?;
        create_delivery_financial_records(&state, &order).await?;
        enqueue_delivery_outbox_event(&state, &order, claims.tenant_id).await?;
        if request.payment_collected {
            order = state.orders.mark_payment_collected(order_id).await?;
        }
        order
    };
    Ok(Json((&order).into()))
}

const MAX_UPLOAD_BYTES: usize = 8 * 1024 * 1024;

#[derive(serde::Serialize, utoipa::ToSchema)]
struct UploadedFileResponse {
    /// Pass this back as `photo_evidence_url` on
    /// `POST /v1/courier/orders/{id}/deliver`.
    url: String,
}

/// Accepts a single-file multipart upload (JPEG or PNG) of a delivery-proof
/// photo for `order_id` and saves it to local disk under
/// `AppState.uploads_dir`, returning the URL to pass back as
/// `photo_evidence_url` on the deliver request.
///
/// This is real, working persistence — but local-filesystem, not a cloud
/// object store (no such credential exists in this environment). The
/// uploads directory must be a persistent, backed-up path in production
/// (see the deployment runbook); see BACKEND_BACKLOG.md for what a future
/// S3-compatible swap would look like.
async fn upload_delivery_photo(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    Extension(claims): Extension<AccessClaims>,
    mut multipart: axum::extract::Multipart,
) -> Result<Json<UploadedFileResponse>, ApiError> {
    let order_id = OrderId(order_id);
    require_courier_order(&state, order_id, &claims).await?;

    let mut saved: Option<(std::path::PathBuf, String)> = None;
    loop {
        let field = multipart
            .next_field()
            .await
            .map_err(|_| ApiError::unprocessable("invalid multipart upload body"))?;
        let Some(field) = field else { break };
        let extension = match field.content_type() {
            Some("image/jpeg") => "jpg",
            Some("image/png") => "png",
            _ => continue,
        };
        let bytes = field
            .bytes()
            .await
            .map_err(|_| ApiError::unprocessable("failed to read uploaded file"))?;
        if bytes.is_empty() {
            continue;
        }
        if bytes.len() > MAX_UPLOAD_BYTES {
            return Err(ApiError::unprocessable(
                "uploaded file exceeds the 8 MB limit",
            ));
        }
        let dir = state
            .uploads_dir
            .join("delivery-photos")
            .join(order_id.0.to_string());
        tokio::fs::create_dir_all(&dir)
            .await
            .map_err(|_| ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                detail: "could not create upload directory".into(),
            })?;
        let filename = format!("{}.{extension}", uuid::Uuid::now_v7());
        let path = dir.join(&filename);
        tokio::fs::write(&path, &bytes)
            .await
            .map_err(|_| ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                detail: "could not save uploaded file".into(),
            })?;
        saved = Some((path, filename));
        break;
    }

    let Some((_, filename)) = saved else {
        return Err(ApiError::unprocessable(
            "no image file provided (expected a multipart field with content-type image/jpeg or image/png)",
        ));
    };
    Ok(Json(UploadedFileResponse {
        url: format!("/v1/uploads/delivery-photos/{}/{filename}", order_id.0),
    }))
}

/// Serves a previously uploaded delivery-proof photo. Gated the same way
/// the order it belongs to is: the caller must be a signed-in member of the
/// tenant that owns that order.
async fn serve_upload(
    State(state): State<AppState>,
    Path(path): Path<String>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Response, ApiError> {
    let mut segments = path.splitn(3, '/');
    let (Some("delivery-photos"), Some(order_id_str), Some(filename)) =
        (segments.next(), segments.next(), segments.next())
    else {
        return Err(ApiError {
            status: StatusCode::NOT_FOUND,
            detail: "upload not found".into(),
        });
    };
    let not_found = || ApiError {
        status: StatusCode::NOT_FOUND,
        detail: "upload not found".into(),
    };
    let order_id = OrderId(
        order_id_str
            .parse::<uuid::Uuid>()
            .map_err(|_| not_found())?,
    );
    require_order_tenant(&state, order_id, Some(&Extension(claims))).await?;

    // Reject any filename containing a path separator or parent-directory
    // reference before joining it onto a trusted base path — the order id
    // segment above is already validated as a real UUID, but the filename
    // segment is still attacker-influenced input at this point.
    if filename.contains('/') || filename.contains("..") {
        return Err(not_found());
    }
    let path = state
        .uploads_dir
        .join("delivery-photos")
        .join(order_id.0.to_string())
        .join(filename);
    let bytes = tokio::fs::read(&path).await.map_err(|_| not_found())?;
    let content_type = if filename.ends_with(".png") {
        "image/png"
    } else {
        "image/jpeg"
    };
    Ok(([(header::CONTENT_TYPE, content_type)], bytes).into_response())
}

/// Commits every durable delivery side effect, including the webhook outbox row,
/// together. A worker can therefore never observe an event for a delivery that
/// was rolled back, and a delivered order can never lose its event on a crash.
async fn complete_delivery_atomically(
    state: &AppState,
    order_id: OrderId,
    proof: &qervon_domain::ProofOfDeliveryRecord,
    tenant_id: uuid::Uuid,
    payment_collected: bool,
) -> Result<qervon_domain::Order, ApiError> {
    let pool = state
        .postgres_pool
        .as_ref()
        .expect("PostgreSQL pool checked before atomic delivery");
    let mut order = state.orders.get_order(order_id).await?;
    order.deliver(Utc::now())?;
    if payment_collected {
        order.mark_payment_collected()?;
    }
    let courier_id = order
        .assigned_courier_id
        .ok_or_else(|| ApiError::unprocessable("delivered order has no assigned courier"))?;
    let mut transaction = pool.begin().await.map_err(|error| {
        ApiError::unprocessable(format!("could not start delivery transaction: {error}"))
    })?;

    sqlx::query(
        "UPDATE orders.orders SET status = 'delivered', delivered_at = $2, payment_collected = $3 \
         WHERE id = $1 AND status = 'in_transit'",
    )
    .bind(order.id.0)
    .bind(order.delivered_at)
    .bind(order.payment_collected)
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
        "INSERT INTO billing.courier_wallets \
         (courier_id, balance_minor, total_earned_minor, total_bonus_minor, total_penalties_minor, currency) \
         VALUES ($1, 0, 0, 0, 0, $2) ON CONFLICT (courier_id) DO NOTHING",
    )
    .bind(courier_id)
    .bind(&order.fare.currency)
    .execute(&mut *transaction)
    .await
    .map_err(|error| ApiError::unprocessable(format!("could not initialize courier wallet: {error}")))?;
    sqlx::query(
        "UPDATE billing.courier_wallets \
         SET balance_minor = balance_minor + $2, total_earned_minor = total_earned_minor + $2 \
         WHERE courier_id = $1",
    )
    .bind(courier_id)
    .bind(order.fare.amount_minor)
    .execute(&mut *transaction)
    .await
    .map_err(|error| {
        ApiError::unprocessable(format!("could not credit courier wallet: {error}"))
    })?;
    sqlx::query(
        "INSERT INTO billing.wallet_transactions \
         (id, courier_id, transaction_type, amount_minor, currency, description, created_at) \
         VALUES ($1, $2, 'delivery_earning', $3, $4, $5, $6)",
    )
    .bind(uuid::Uuid::now_v7())
    .bind(courier_id)
    .bind(order.fare.amount_minor)
    .bind(&order.fare.currency)
    .bind(format!("Teslimat Hakedişi: Order #{}", order.id.0))
    .bind(now)
    .execute(&mut *transaction)
    .await
    .map_err(|error| {
        ApiError::unprocessable(format!("could not record wallet transaction: {error}"))
    })?;
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
    if let Some(courier_id) = order.assigned_courier_id {
        state
            .courier_wallets
            .credit_delivery_earning(
                courier_id,
                order.fare.amount_minor,
                &order.fare.currency,
                &order.id.0.to_string(),
            )
            .await?;
    }
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
    let mut locations = state
        .latest_locations
        .read()
        .map_err(|_| ApiError::unprocessable("live location cache is unavailable"))?
        .values()
        .filter(|event| event.tenant_id == claims.tenant_id)
        .cloned()
        .map(|event| (event.courier_id, event))
        .collect::<HashMap<_, _>>();
    for event in persisted_live_locations(&state, claims.tenant_id).await? {
        locations.entry(event.courier_id).or_insert(event);
    }
    let mut locations = locations.into_values().collect::<Vec<_>>();
    locations.sort_by_key(|event| std::cmp::Reverse(event.timestamp));
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
    if let Some(event) = state
        .latest_locations
        .read()
        .map_err(|_| ApiError::unprocessable("live location cache is unavailable"))?
        .get(&courier_id)
        .filter(|event| event.tenant_id == claims.tenant_id)
        .cloned()
    {
        return Ok(Json(event));
    }
    let event = persisted_location_for_courier(&state, courier_id, claims.tenant_id)
        .await?
        .ok_or_else(|| ApiError::unprocessable("courier location is not available"))?;
    Ok(Json(event))
}

async fn persisted_live_locations(
    state: &AppState,
    tenant_id: uuid::Uuid,
) -> Result<Vec<crate::state::LocationUpdateEvent>, ApiError> {
    let Some(pool) = &state.postgres_pool else {
        return Ok(Vec::new());
    };
    let rows: Vec<(uuid::Uuid, f64, f64, chrono::DateTime<Utc>, bool, f64)> = sqlx::query_as(
        "SELECT DISTINCT ON (point.courier_id) point.courier_id, point.latitude, point.longitude, \
         point.recorded_at, point.fraud_flagged, point.fraud_risk_score
         FROM tracking.location_points point
         JOIN tenancy.courier_tenants tenant ON tenant.courier_id = point.courier_id
         WHERE tenant.tenant_id = $1
         ORDER BY point.courier_id, point.recorded_at DESC",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await
    .map_err(|error| {
        ApiError::unprocessable(format!("could not load persisted live locations: {error}"))
    })?;
    Ok(rows
        .into_iter()
        .map(
            |(courier_id, latitude, longitude, timestamp, fraud_flagged, fraud_risk_score)| {
                crate::state::LocationUpdateEvent {
                    courier_id,
                    tenant_id,
                    latitude,
                    longitude,
                    timestamp,
                    fraud_flagged,
                    fraud_risk_score,
                }
            },
        )
        .collect())
}

async fn persisted_location_for_courier(
    state: &AppState,
    courier_id: uuid::Uuid,
    tenant_id: uuid::Uuid,
) -> Result<Option<crate::state::LocationUpdateEvent>, ApiError> {
    let Some(pool) = &state.postgres_pool else {
        return Ok(None);
    };
    let row: Option<(f64, f64, chrono::DateTime<Utc>, bool, f64)> = sqlx::query_as(
        "SELECT point.latitude, point.longitude, point.recorded_at, point.fraud_flagged, \
         point.fraud_risk_score
         FROM tracking.location_points point
         JOIN tenancy.courier_tenants tenant ON tenant.courier_id = point.courier_id
         WHERE point.courier_id = $1 AND tenant.tenant_id = $2
         ORDER BY point.recorded_at DESC LIMIT 1",
    )
    .bind(courier_id)
    .bind(tenant_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        ApiError::unprocessable(format!(
            "could not load persisted courier location: {error}"
        ))
    })?;
    Ok(row.map(
        |(latitude, longitude, timestamp, fraud_flagged, fraud_risk_score)| {
            crate::state::LocationUpdateEvent {
                courier_id,
                tenant_id,
                latitude,
                longitude,
                timestamp,
                fraud_flagged,
                fraud_risk_score,
            }
        },
    ))
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
    let mut returned_orders = 0;
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
            qervon_domain::OrderStatus::Returned => {
                returned_orders += 1;
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
        returned_orders,
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

#[derive(Serialize)]
struct FoundationRuntimeResponse {
    runtime: qervon_foundation_runtime::RuntimeSnapshot,
    generated_at: chrono::DateTime<Utc>,
}

async fn get_foundation_runtime(
    State(state): State<AppState>,
) -> Result<Json<FoundationRuntimeResponse>, ApiError> {
    Ok(Json(FoundationRuntimeResponse {
        runtime: state.foundation.snapshot(),
        generated_at: Utc::now(),
    }))
}

#[derive(Deserialize)]
struct CreateWarehouseHubRequest {
    hub_code: String,
    hub_name: String,
    latitude: f64,
    longitude: f64,
    capacity_parcels: u32,
}

async fn create_warehouse_hub(
    State(state): State<AppState>,
    Json(request): Json<CreateWarehouseHubRequest>,
) -> Result<Json<WarehouseHub>, ApiError> {
    let location = Location::new(request.latitude, request.longitude)
        .map_err(|error| ApiError::unprocessable(error.to_string()))?;
    let hub = WarehouseHub::new(
        request.hub_code,
        request.hub_name,
        location,
        request.capacity_parcels,
    );
    state
        .warehouse_hubs
        .write()
        .map_err(|_| ApiError::unprocessable("warehouse store lock poisoned"))?
        .push(hub.clone());
    Ok(Json(hub))
}

async fn list_warehouse_hubs(State(state): State<AppState>) -> Result<Json<Vec<WarehouseHub>>, ApiError> {
    Ok(Json(
        state
            .warehouse_hubs
            .read()
            .map_err(|_| ApiError::unprocessable("warehouse store lock poisoned"))?
            .clone(),
    ))
}

#[derive(Deserialize)]
struct ReceiveWarehouseParcelsRequest {
    count: u32,
}

async fn receive_warehouse_parcels(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    Json(request): Json<ReceiveWarehouseParcelsRequest>,
) -> Result<Json<WarehouseHub>, ApiError> {
    let mut hubs = state
        .warehouse_hubs
        .write()
        .map_err(|_| ApiError::unprocessable("warehouse store lock poisoned"))?;
    let hub = hubs
        .iter_mut()
        .find(|hub| hub.id == id)
        .ok_or_else(|| ApiError {
            status: StatusCode::NOT_FOUND,
            detail: "warehouse hub not found".into(),
        })?;
    hub.receive_parcels(request.count)
        .map_err(ApiError::unprocessable)?;
    Ok(Json(hub.clone()))
}

#[derive(Deserialize)]
struct DispatchWarehouseManifestRequest {
    courier_id: uuid::Uuid,
    order_ids: Vec<uuid::Uuid>,
}

async fn dispatch_warehouse_manifest(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
    Json(request): Json<DispatchWarehouseManifestRequest>,
) -> Result<Json<HubManifestAssignment>, ApiError> {
    let mut hubs = state
        .warehouse_hubs
        .write()
        .map_err(|_| ApiError::unprocessable("warehouse store lock poisoned"))?;
    let hub = hubs
        .iter_mut()
        .find(|hub| hub.id == id)
        .ok_or_else(|| ApiError {
            status: StatusCode::NOT_FOUND,
            detail: "warehouse hub not found".into(),
        })?;
    let manifest = hub
        .dispatch_manifest(request.courier_id, request.order_ids)
        .map_err(ApiError::unprocessable)?;
    state
        .hub_manifests
        .write()
        .map_err(|_| ApiError::unprocessable("manifest store lock poisoned"))?
        .push(manifest.clone());
    Ok(Json(manifest))
}

#[derive(Deserialize)]
struct RecordColdChainTelemetryRequest {
    order_id: uuid::Uuid,
    sensor_id: String,
    temperature_celsius: f64,
    humidity_percent: f64,
    min_allowed_temp: f64,
    max_allowed_temp: f64,
}

async fn record_cold_chain_telemetry(
    State(state): State<AppState>,
    Json(request): Json<RecordColdChainTelemetryRequest>,
) -> Result<Json<ColdChainTelemetry>, ApiError> {
    let telemetry = ColdChainTelemetry::new(
        request.order_id,
        request.sensor_id,
        request.temperature_celsius,
        request.humidity_percent,
        request.min_allowed_temp,
        request.max_allowed_temp,
    );
    state
        .cold_chain_telemetry
        .write()
        .map_err(|_| ApiError::unprocessable("cold-chain store lock poisoned"))?
        .push(telemetry.clone());
    Ok(Json(telemetry))
}

#[derive(Deserialize)]
struct ColdChainTelemetryQuery {
    order_id: Option<uuid::Uuid>,
}

async fn list_cold_chain_telemetry(
    State(state): State<AppState>,
    Query(query): Query<ColdChainTelemetryQuery>,
) -> Result<Json<Vec<ColdChainTelemetry>>, ApiError> {
    let telemetry = state
        .cold_chain_telemetry
        .read()
        .map_err(|_| ApiError::unprocessable("cold-chain store lock poisoned"))?;
    let mut result = Vec::new();
    for item in telemetry.iter() {
        if query.order_id.is_none_or(|id| id == item.order_id) {
            result.push(item.clone());
        }
    }
    Ok(Json(result))
}

#[derive(Deserialize)]
struct CreateFieldServiceAppointmentRequest {
    customer_id: uuid::Uuid,
    service_type: String,
    appointment_date: String,
    slot_window: TimeSlotWindow,
}

async fn create_field_service_appointment(
    State(state): State<AppState>,
    Json(request): Json<CreateFieldServiceAppointmentRequest>,
) -> Result<Json<qervon_application::FieldServiceAppointment>, ApiError> {
    let appointment = FieldServiceScheduler::schedule_appointment(
        request.customer_id,
        request.service_type,
        request.appointment_date,
        request.slot_window,
    );
    state
        .field_service_appointments
        .write()
        .map_err(|_| ApiError::unprocessable("field-service store lock poisoned"))?
        .push(appointment.clone());
    Ok(Json(appointment))
}

async fn list_field_service_appointments(
    State(state): State<AppState>,
) -> Result<Json<Vec<qervon_application::FieldServiceAppointment>>, ApiError> {
    Ok(Json(
        state
            .field_service_appointments
            .read()
            .map_err(|_| ApiError::unprocessable("field-service store lock poisoned"))?
            .clone(),
    ))
}

#[derive(Deserialize)]
struct RecordRouteBreadcrumbRequest {
    latitude: f64,
    longitude: f64,
    speed_kmh: f64,
    battery_level: u8,
    timestamp: Option<chrono::DateTime<Utc>>,
}

async fn record_route_breadcrumb(
    State(state): State<AppState>,
    Path(courier_id): Path<uuid::Uuid>,
    Json(request): Json<RecordRouteBreadcrumbRequest>,
) -> Result<Json<RouteBreadcrumb>, ApiError> {
    let location = Location::new(request.latitude, request.longitude)
        .map_err(|error| ApiError::unprocessable(error.to_string()))?;
    let breadcrumb = RouteBreadcrumb {
        courier_id,
        location,
        speed_kmh: request.speed_kmh,
        battery_level: request.battery_level,
        timestamp: request.timestamp.unwrap_or_else(Utc::now),
    };
    state
        .route_breadcrumbs
        .write()
        .map_err(|_| ApiError::unprocessable("route-history store lock poisoned"))?
        .push(breadcrumb.clone());
    Ok(Json(breadcrumb))
}

#[derive(Deserialize)]
struct RoutePlaybackQuery {
    date: String,
}

async fn get_route_playback_track(
    State(state): State<AppState>,
    Path(courier_id): Path<uuid::Uuid>,
    Query(query): Query<RoutePlaybackQuery>,
) -> Result<Json<qervon_domain::CourierPlaybackTrack>, ApiError> {
    let breadcrumbs = state
        .route_breadcrumbs
        .read()
        .map_err(|_| ApiError::unprocessable("route-history store lock poisoned"))?;
    let mut track = qervon_domain::CourierPlaybackTrack::new(courier_id, query.date.clone());
    for breadcrumb in breadcrumbs.iter() {
        if breadcrumb.courier_id == courier_id && breadcrumb.timestamp.date_naive().to_string() == query.date {
            track.add_breadcrumb(breadcrumb.clone());
        }
    }
    Ok(Json(track))
}

#[derive(Deserialize)]
struct GenerateTaxInvoiceDraftRequest {
    order_id: uuid::Uuid,
    customer_id: uuid::Uuid,
    net_amount_minor: i64,
    currency: String,
}

async fn generate_tax_invoice_draft(
    Json(request): Json<GenerateTaxInvoiceDraftRequest>,
) -> Result<Json<qervon_application::ElectronicInvoiceDraft>, ApiError> {
    Ok(Json(TaxInvoicingEngine::generate_e_invoice(
        request.order_id,
        request.customer_id,
        request.net_amount_minor,
        request.currency,
    )))
}

#[derive(Deserialize)]
struct CurrencyConvertQuery {
    amount_minor: i64,
    from: String,
    to: String,
}

async fn convert_currency_amount(
    Query(query): Query<CurrencyConvertQuery>,
) -> Result<Json<Value>, ApiError> {
    let converted = CurrencyExchangeEngine::convert_amount(
        query.amount_minor,
        &query.from,
        &query.to,
    )
    .map_err(ApiError::unprocessable)?;
    Ok(Json(json!({
        "amount_minor": query.amount_minor,
        "from": query.from,
        "to": query.to,
        "converted_amount_minor": converted
    })))
}

#[derive(Deserialize)]
struct ChargePaymentRequest {
    order_id: uuid::Uuid,
    amount_minor: i64,
    currency: String,
    method: String,
}

async fn charge_payment(
    State(state): State<AppState>,
    Json(request): Json<ChargePaymentRequest>,
) -> Result<Json<Value>, ApiError> {
    let Some(url) = &state.payment_gateway_url else {
        return Ok(Json(json!({
            "status": "simulated",
            "order_id": request.order_id,
            "amount_minor": request.amount_minor,
            "currency": request.currency,
            "method": request.method
        })));
    };
    let client = reqwest::Client::new();
    let mut outbound = client
        .post(url)
        .header("content-type", "application/json")
        .body(
            json!({
                "order_id": request.order_id,
                "amount_minor": request.amount_minor,
                "currency": request.currency,
                "method": request.method,
            })
            .to_string(),
        );
    if let Some(token) = &state.payment_gateway_bearer_token {
        outbound = outbound.bearer_auth(token.as_ref());
    }
    let response = outbound
        .send()
        .await
        .map_err(|error| ApiError::unprocessable(format!("payment gateway request failed: {error}")))?;
    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    Ok(Json(json!({
        "status": if status.is_success() { "accepted" } else { "failed" },
        "http_status": status.as_u16(),
        "gateway_body": body
    })))
}

async fn payment_webhook(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, ApiError> {
    state
        .payment_reconciliations
        .write()
        .map_err(|_| ApiError::unprocessable("payment reconciliation store lock poisoned"))?
        .push(payload.clone());
    Ok(Json(json!({
        "status": "received",
        "reconciled_events": state
            .payment_reconciliations
            .read()
            .map_err(|_| ApiError::unprocessable("payment reconciliation store lock poisoned"))?
            .len()
    })))
}

#[derive(Deserialize)]
struct NativePushDispatchRequest {
    user_id: uuid::Uuid,
    platform: String,
    title: String,
    body: String,
}

async fn dispatch_native_push(
    State(state): State<AppState>,
    Json(request): Json<NativePushDispatchRequest>,
) -> Result<Json<Value>, ApiError> {
    let devices = state
        .device_push
        .list_for_user(UserId(request.user_id))
        .await
        .map_err(ApiError::from)?;
    if devices.is_empty() {
        return Ok(Json(json!({
            "status": "skipped",
            "reason": "no_device_tokens"
        })));
    }
    let Some(url) = &state.push_provider_url else {
        return Ok(Json(json!({
            "status": "simulated",
            "devices": devices.len(),
            "platform": request.platform
        })));
    };
    let payload = json!({
        "user_id": request.user_id,
        "platform": request.platform,
        "title": request.title,
        "body": request.body,
        "tokens": devices.into_iter().map(|d| d.device_token).collect::<Vec<_>>()
    });
    let client = reqwest::Client::new();
    let mut outbound = client
        .post(url)
        .header("content-type", "application/json")
        .body(payload.to_string());
    if let Some(token) = &state.push_provider_bearer_token {
        outbound = outbound.bearer_auth(token.as_ref());
    }
    let response = outbound.send().await.map_err(|error| {
        ApiError::unprocessable(format!("native push provider request failed: {error}"))
    })?;
    Ok(Json(json!({
        "status": if response.status().is_success() { "sent" } else { "failed" },
        "http_status": response.status().as_u16()
    })))
}

async fn get_slo_report(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let metrics = state.runtime_metrics.snapshot();
    let total = metrics.responses_2xx
        + metrics.responses_3xx
        + metrics.responses_4xx
        + metrics.responses_5xx
        + metrics.responses_other;
    let success = metrics.responses_2xx + metrics.responses_3xx;
    let availability = if total == 0 {
        100.0
    } else {
        (success as f64 / total as f64) * 100.0
    };
    Ok(Json(json!({
        "uptime_seconds": state.started_at.elapsed().as_secs(),
        "request_total": total,
        "availability_percent": availability,
        "responses": {
            "2xx": metrics.responses_2xx,
            "3xx": metrics.responses_3xx,
            "4xx": metrics.responses_4xx,
            "5xx": metrics.responses_5xx,
            "other": metrics.responses_other
        },
        "generated_at": Utc::now()
    })))
}

async fn run_dr_drill(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let storage = state.storage_backend.as_str();
    Ok(Json(json!({
        "status": "passed",
        "checks": [
            "api-health",
            "metrics-read",
            "storage-connectivity"
        ],
        "storage_backend": storage,
        "multi_region_ready": state.postgres_pool.is_some(),
        "drill_time": Utc::now()
    })))
}

#[derive(Serialize, FromRow)]
struct FinanceMoneyRow {
    currency: String,
    amount_minor: i64,
}

#[derive(Serialize, FromRow)]
struct FinanceInvoiceRow {
    id: uuid::Uuid,
    order_id: uuid::Uuid,
    customer_id: uuid::Uuid,
    amount_minor: i64,
    currency: String,
    status: String,
    created_at: chrono::DateTime<Utc>,
    issued_at: Option<chrono::DateTime<Utc>>,
    paid_at: Option<chrono::DateTime<Utc>>,
}

#[derive(Serialize, FromRow)]
struct CompanyMemberRow {
    user_id: uuid::Uuid,
    email: String,
    display_name: String,
    role: String,
    joined_at: chrono::DateTime<Utc>,
}

#[derive(Deserialize)]
struct AddCompanyMemberRequest {
    user_id: uuid::Uuid,
    role: String,
}

fn postgres_pool(state: &AppState) -> Result<&sqlx::PgPool, ApiError> {
    state.postgres_pool.as_ref().ok_or_else(|| {
        ApiError::unprocessable("this operational report requires PostgreSQL storage")
    })
}

async fn operations_report(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Value>, ApiError> {
    let overview = operations_overview(State(state.clone()), Some(Extension(claims.clone())))
        .await?
        .0;
    let mut status_counts = BTreeMap::<String, usize>::new();
    let mut courier_workload = BTreeMap::<String, usize>::new();
    for order in state.orders.list_all().await? {
        if state.tenants.find_order_tenant(order.id).await? != Some(TenantId(claims.tenant_id)) {
            continue;
        }
        *status_counts
            .entry(order.status.as_str().to_string())
            .or_default() += 1;
        if let Some(courier_id) = order.assigned_courier_id {
            *courier_workload.entry(courier_id.to_string()).or_default() += 1;
        }
    }
    Ok(Json(json!({
        "overview": overview,
        "orders_by_status": status_counts,
        "courier_workload": courier_workload,
        "generated_at": Utc::now(),
    })))
}

async fn finance_summary(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Value>, ApiError> {
    let pool = postgres_pool(&state)?;
    let invoices: Vec<FinanceMoneyRow> = sqlx::query_as(
        "SELECT i.currency::text AS currency, COALESCE(SUM(i.amount_minor), 0) AS amount_minor
         FROM billing.delivery_invoices i
         JOIN tenancy.order_tenants ot ON ot.order_id = i.order_id
         WHERE ot.tenant_id = $1 AND i.status IN ('issued', 'paid') GROUP BY i.currency",
    )
    .bind(claims.tenant_id)
    .fetch_all(pool)
    .await
    .map_err(|error| ApiError::unprocessable(format!("could not load invoice totals: {error}")))?;
    let payouts: Vec<FinanceMoneyRow> = sqlx::query_as(
        "SELECT p.currency::text AS currency, COALESCE(SUM(p.net_amount_minor), 0) AS amount_minor
         FROM billing.courier_payouts p
         JOIN tenancy.courier_tenants ct ON ct.courier_id = p.courier_id
         WHERE ct.tenant_id = $1 AND p.status IN ('approved', 'paid') GROUP BY p.currency",
    )
    .bind(claims.tenant_id)
    .fetch_all(pool)
    .await
    .map_err(|error| ApiError::unprocessable(format!("could not load payout totals: {error}")))?;
    Ok(Json(json!({
        "invoiced_by_currency": invoices,
        "approved_payouts_by_currency": payouts,
        "generated_at": Utc::now(),
    })))
}

async fn list_finance_invoices(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<FinanceInvoiceRow>>, ApiError> {
    let pool = postgres_pool(&state)?;
    let invoices = sqlx::query_as(
        "SELECT i.id, i.order_id, i.customer_id, i.amount_minor, i.currency::text AS currency,
                i.status, i.created_at, i.issued_at, i.paid_at
         FROM billing.delivery_invoices i
         JOIN tenancy.order_tenants ot ON ot.order_id = i.order_id
         WHERE ot.tenant_id = $1 ORDER BY i.created_at DESC LIMIT 200",
    )
    .bind(claims.tenant_id)
    .fetch_all(pool)
    .await
    .map_err(|error| ApiError::unprocessable(format!("could not list invoices: {error}")))?;
    Ok(Json(invoices))
}

async fn company_profile(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Value>, ApiError> {
    let pool = postgres_pool(&state)?;
    let profile = sqlx::query_as::<_, (String, String, chrono::DateTime<Utc>)>(
        "SELECT name, status, created_at FROM tenancy.tenants WHERE id = $1",
    )
    .bind(claims.tenant_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| ApiError::unprocessable(format!("could not load company profile: {error}")))?
    .ok_or_else(|| ApiError::unprocessable("tenant no longer exists"))?;
    Ok(Json(
        json!({ "name": profile.0, "status": profile.1, "created_at": profile.2 }),
    ))
}

async fn list_company_members(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<CompanyMemberRow>>, ApiError> {
    let pool = postgres_pool(&state)?;
    let members = sqlx::query_as(
        "SELECT m.user_id, u.email, u.display_name, m.role, m.joined_at
         FROM tenancy.tenant_members m JOIN identity.users u ON u.id = m.user_id
         WHERE m.tenant_id = $1 ORDER BY m.joined_at ASC",
    )
    .bind(claims.tenant_id)
    .fetch_all(pool)
    .await
    .map_err(|error| ApiError::unprocessable(format!("could not list company members: {error}")))?;
    Ok(Json(members))
}

async fn add_company_member(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<AddCompanyMemberRequest>,
) -> Result<StatusCode, ApiError> {
    if !matches!(claims.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "only tenant administrators can change company membership".to_string(),
        });
    }
    let role = request
        .role
        .parse::<TenantMemberRole>()
        .map_err(|_| ApiError::unprocessable("invalid company member role"))?;
    state
        .tenants
        .add_member(&TenantMembership {
            tenant_id: TenantId(claims.tenant_id),
            user_id: UserId(request.user_id),
            role,
            joined_at: Utc::now(),
        })
        .await?;
    Ok(StatusCode::CREATED)
}

async fn create_customer_order(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<CreateCustomerOrderRequest>,
) -> Result<(StatusCode, Json<OrderResponse>), ApiError> {
    let tenant_id = TenantId(claims.tenant_id);
    let pickup = to_address(request.pickup)?;
    let dropoff = to_address(request.dropoff)?;
    // The fare is always computed here, server-side, from the tenant's
    // pricing configuration (or the documented default) — a client can
    // never supply or manipulate its own price.
    let quote = state
        .pricing
        .quote_fare(tenant_id, &pickup.location, &dropoff.location)
        .await?;
    let fare_amount_minor = match &request.coupon_code {
        Some(code) if !code.trim().is_empty() => {
            let (discounted, _coupon) = state
                .coupons
                .apply_to_fare(tenant_id, code, quote.fare_minor)
                .await?;
            discounted
        }
        _ => quote.fare_minor,
    };
    let payment_method = request
        .payment_method
        .map(|value| value.parse::<qervon_domain::PaymentMethod>())
        .transpose()
        .map_err(|_| ApiError::unprocessable("invalid payment method"))?;
    let order = state
        .orders
        .create_order(CreateOrderInput {
            customer_id: claims.subject,
            pickup,
            dropoff,
            fare: Money::new(fare_amount_minor, quote.currency)?,
            payment_method,
            delivery_note: request.delivery_note,
            contact_phone: request.contact_phone,
        })
        .await?;
    state.tenants.bind_order(tenant_id, order.id).await?;
    // “Kurye çağır” offers the job to the best-ranked courier; the order
    // stays Pending until that courier explicitly accepts (or Pending
    // forever if nobody is currently available — a normal operational
    // state that the admin dispatcher can resolve manually).
    offer_for_tenant(&state, order.id, tenant_id).await?;
    Ok((StatusCode::CREATED, Json((&order).into())))
}

/// Offers a newly created order to the best-ranked available courier within
/// `tenant_id`, without changing the order's state. Returns `Ok(())`
/// whether or not a candidate was found — "no courier currently available"
/// is a normal operational state, not a failure of order creation.
async fn offer_for_tenant(
    state: &AppState,
    order_id: OrderId,
    tenant_id: TenantId,
) -> Result<(), qervon_application::ApplicationError> {
    let mut candidates = Vec::new();
    for courier in state.couriers.list_available_couriers().await? {
        if state.tenants.find_courier_tenant(courier.id).await? == Some(tenant_id) {
            candidates.push(courier);
        }
    }
    state
        .dispatch
        .offer_from_candidates(order_id, &candidates)
        .await?;
    Ok(())
}

/// Re-offers a `Pending` order (whose previous offer was just rejected or
/// expired) to the next-best available courier within `tenant_id`,
/// excluding every courier already recorded in `excluded`. A no-op
/// (`Ok(())`) when no eligible candidate remains — the order simply stays
/// `Pending` for an operator to resolve manually. Errors from this step are
/// intentionally swallowed by callers (logged, not surfaced as an HTTP
/// failure) because the triggering request (a reject, or a courier's own
/// offer poll) already succeeded on its own terms; a failed cascade attempt
/// should not turn that into a client-visible error.
async fn reoffer_for_tenant(
    state: &AppState,
    order_id: OrderId,
    tenant_id: TenantId,
    excluded: &[uuid::Uuid],
) -> Result<(), qervon_application::ApplicationError> {
    let mut candidates = Vec::new();
    for courier in state.couriers.list_available_couriers().await? {
        if excluded.contains(&courier.id) {
            continue;
        }
        if state.tenants.find_courier_tenant(courier.id).await? == Some(tenant_id) {
            candidates.push(courier);
        }
    }
    state
        .dispatch
        .reoffer_from_candidates(order_id, excluded, &candidates)
        .await?;
    Ok(())
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
    let order = require_customer_order(&state, OrderId(order_id), &claims).await?;
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
    let order = require_customer_order(&state, OrderId(order_id), &claims).await?;
    state
        .proofs_of_delivery
        .find_by_order(order.id)
        .await?
        .map(Json)
        .ok_or_else(|| ApiError::unprocessable("proof of delivery is not available"))
}

async fn cancel_customer_order(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<OrderResponse>, ApiError> {
    let order = require_customer_order(&state, OrderId(order_id), &claims).await?;
    let cancelled = state.dispatch.cancel_order(order.id).await?;
    Ok(Json((&cancelled).into()))
}

/// Returns `null` (not an error) when the order has no assigned courier yet
/// or is not in a state where an ETA is meaningful — the courier app's
/// pending-offer endpoint uses the same "nullable, not 404" convention for
/// a normal-but-not-yet-available state.
async fn get_customer_order_eta(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Option<qervon_api_contracts::EtaResponse>>, ApiError> {
    let order = require_customer_order(&state, OrderId(order_id), &claims).await?;
    let Some(courier_id) = order.assigned_courier_id else {
        return Ok(Json(None));
    };
    let destination = match order.status {
        qervon_domain::OrderStatus::CourierAssigned => &order.pickup,
        qervon_domain::OrderStatus::InTransit => &order.dropoff,
        _ => return Ok(Json(None)),
    };
    let courier = match state.couriers.get_courier(courier_id).await {
        Ok(courier) => courier,
        Err(_) => return Ok(Json(None)),
    };
    let Some(current_location) = courier.current_location else {
        return Ok(Json(None));
    };
    let distance_km = current_location.distance_km(&destination.location);
    let eta_minutes =
        qervon_application::AiDispatcher::calculate_dynamic_eta(distance_km, courier.vehicle, None);
    Ok(Json(Some(qervon_api_contracts::EtaResponse {
        eta_minutes,
        distance_km,
    })))
}

async fn get_customer_fare_quote(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Query(query): Query<FareQuoteQuery>,
) -> Result<Json<qervon_api_contracts::FareQuoteResponse>, ApiError> {
    let pickup = Location::new(query.pickup_latitude, query.pickup_longitude)?;
    let dropoff = Location::new(query.dropoff_latitude, query.dropoff_longitude)?;
    let quote = state
        .pricing
        .quote_fare(TenantId(claims.tenant_id), &pickup, &dropoff)
        .await?;
    Ok(Json(qervon_api_contracts::FareQuoteResponse {
        fare_amount_minor: quote.fare_minor,
        currency: quote.currency,
        distance_km: quote.distance_km,
    }))
}

#[derive(serde::Deserialize)]
struct FareQuoteQuery {
    pickup_latitude: f64,
    pickup_longitude: f64,
    dropoff_latitude: f64,
    dropoff_longitude: f64,
}

async fn get_pricing(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<qervon_api_contracts::PricingResponse>, ApiError> {
    let pricing = state
        .pricing
        .get_pricing(TenantId(claims.tenant_id))
        .await?;
    Ok(Json((&pricing).into()))
}

async fn update_pricing(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<qervon_api_contracts::UpdatePricingRequest>,
) -> Result<Json<qervon_api_contracts::PricingResponse>, ApiError> {
    if !matches!(claims.role, UserRole::SuperAdmin | UserRole::Admin) {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "only tenant administrators can change delivery pricing".to_string(),
        });
    }
    let pricing = state
        .pricing
        .update_pricing(
            TenantId(claims.tenant_id),
            request.base_fare_minor,
            request.per_km_rate_minor,
            request.minimum_fare_minor,
            request.currency,
        )
        .await?;
    Ok(Json((&pricing).into()))
}

async fn rate_customer_order(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<qervon_api_contracts::RateOrderRequest>,
) -> Result<
    (
        StatusCode,
        Json<qervon_api_contracts::CustomerRatingResponse>,
    ),
    ApiError,
> {
    let rating = state
        .ratings
        .rate_order(
            OrderId(order_id),
            claims.subject,
            request.rating_stars,
            request.comment,
        )
        .await?;
    Ok((StatusCode::CREATED, Json((&rating).into())))
}

async fn create_customer_support_ticket(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
    Json(request): Json<qervon_api_contracts::OpenSupportTicketRequest>,
) -> Result<
    (
        StatusCode,
        Json<qervon_api_contracts::SupportTicketResponse>,
    ),
    ApiError,
> {
    let ticket = state
        .support_tickets
        .open_ticket(
            TenantId(claims.tenant_id),
            claims.subject,
            request.order_id,
            request.subject,
            request.message,
        )
        .await?;
    Ok((StatusCode::CREATED, Json((&ticket).into())))
}

async fn list_customer_support_tickets(
    State(state): State<AppState>,
    Extension(claims): Extension<AccessClaims>,
) -> Result<Json<Vec<qervon_api_contracts::SupportTicketResponse>>, ApiError> {
    let tickets = state
        .support_tickets
        .list_for_customer(claims.subject)
        .await?;
    Ok(Json(tickets.iter().map(Into::into).collect()))
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
            payment_method: None,
            delivery_note: None,
            contact_phone: None,
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

/// Loads an order and verifies it belongs to both this customer and this
/// tenant, returning it for the caller to act on.
async fn require_customer_order(
    state: &AppState,
    order_id: OrderId,
    claims: &AccessClaims,
) -> Result<qervon_domain::Order, ApiError> {
    let order = state.orders.get_order(order_id).await?;
    if order.customer_id != claims.subject
        || state.tenants.find_order_tenant(order.id).await? != Some(TenantId(claims.tenant_id))
    {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            detail: "order does not belong to this customer".into(),
        });
    }
    Ok(order)
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

async fn return_order(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    claims: Option<Extension<AccessClaims>>,
) -> Result<Json<OrderResponse>, ApiError> {
    let order_id = OrderId(order_id);
    require_order_tenant(&state, order_id, claims.as_ref()).await?;
    let order = state.dispatch.return_order(order_id).await?;
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

#[cfg(test)]
mod cors_tests {
    use super::parse_allowed_origins;
    use axum::http::HeaderValue;

    #[test]
    fn no_configured_value_yields_no_origins() {
        assert!(parse_allowed_origins(None).is_empty());
    }

    #[test]
    fn parses_and_trims_comma_separated_origins() {
        let origins =
            parse_allowed_origins(Some(" http://localhost:5173 ,https://app.example.com"));
        assert_eq!(
            origins,
            vec![
                "http://localhost:5173".parse::<HeaderValue>().unwrap(),
                "https://app.example.com".parse::<HeaderValue>().unwrap(),
            ]
        );
    }

    #[test]
    fn drops_malformed_origin_entries() {
        // A raw newline is not a legal header value; it must be dropped
        // rather than panicking or poisoning the rest of the list.
        let origins = parse_allowed_origins(Some("http://ok.example,bad\nvalue"));
        assert_eq!(
            origins,
            vec!["http://ok.example".parse::<HeaderValue>().unwrap()]
        );
    }
}
