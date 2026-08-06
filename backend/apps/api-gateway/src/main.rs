// =============================================================================
// File:           backend/apps/api-gateway/src/main.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Qervon API Gateway executable entry point.
//
// Specification:
//   QAS-000001 through QAS-000006, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use qervon_api_gateway::{http::router, state::AppState};

const DEFAULT_LISTEN: &str = "0.0.0.0:8080";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "qervon_api_gateway=info,tower_http=info".into()),
        )
        .init();

    let listen = std::env::var("QERVON_LISTEN").unwrap_or_else(|_| DEFAULT_LISTEN.to_string());
    let state = AppState::from_env().await?;

    let app = router(state);
    let listener = tokio::net::TcpListener::bind(&listen).await?;
    tracing::info!(address = %listen, "Qervon API gateway listening");
    axum::serve(listener, app).await?;
    Ok(())
}
