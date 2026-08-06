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
use http_body_util::BodyExt;
use qervon_api_gateway::http::router;
use qervon_api_gateway::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

fn app() -> Router {
    router(AppState::memory())
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
        json!({}),
    )
    .await;
    assert_eq!(status, axum::http::StatusCode::OK);
    assert_eq!(transit["status"], "in_transit");

    let (status, delivered) = request(
        app.clone(),
        "POST",
        &format!("/v1/orders/{order_id}/deliver"),
        json!({}),
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
