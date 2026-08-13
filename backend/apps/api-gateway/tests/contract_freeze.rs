// =============================================================================
// File:           backend/apps/api-gateway/tests/contract_freeze.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-13
// Version:        0.1.0
//
// Description:
//   Baseline API contract freeze checks for anti-regression coverage.
// =============================================================================

use axum::Router;
use http_body_util::BodyExt;
use qervon_api_gateway::http::router;
use qervon_api_gateway::state::AppState;
use serde_json::Value;
use tower::ServiceExt;

fn app() -> Router {
    let mut state = AppState::memory();
    state.api_access_token = Some("test-dev-token".into());
    router(state)
}

async fn get_json(path: &str) -> (axum::http::StatusCode, Value) {
    let request = axum::http::Request::builder()
        .method("GET")
        .uri(path)
        .body(axum::body::Body::empty())
        .expect("request");
    let response = app().oneshot(request).await.expect("response");
    let status = response.status();
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let value = serde_json::from_slice(&body).unwrap_or(Value::Null);
    (status, value)
}

#[tokio::test]
async fn health_and_ready_endpoints_remain_public() {
    let (health_status, health_body) = get_json("/health").await;
    assert_eq!(health_status, axum::http::StatusCode::OK);
    assert_eq!(health_body["status"], "ok");

    let (ready_status, ready_body) = get_json("/ready").await;
    assert_eq!(ready_status, axum::http::StatusCode::OK);
    assert_eq!(ready_body["status"], "ready");
}

#[tokio::test]
async fn openapi_contains_frozen_critical_paths() {
    let request = axum::http::Request::builder()
        .method("GET")
        .uri("/api-docs/openapi.json")
        .body(axum::body::Body::empty())
        .expect("request");
    let response = app().oneshot(request).await.expect("response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    let spec: Value = serde_json::from_slice(&body).expect("openapi json");
    let paths = spec["paths"].as_object().expect("paths object");
    assert!(
        paths.len() >= 5,
        "openapi should expose at least baseline route coverage"
    );
    let joined = paths.keys().cloned().collect::<Vec<_>>().join(" ");
    assert!(
        joined.contains("operations") || joined.contains("orders") || joined.contains("auth"),
        "openapi routes do not include expected baseline domains"
    );
}
