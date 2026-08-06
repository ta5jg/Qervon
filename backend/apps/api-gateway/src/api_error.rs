// =============================================================================
// File:           backend/apps/api-gateway/src/api_error.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   HTTP error mapping from application and domain failures to API responses.
//
// Specification:
//   QAS-000005, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use qervon_api_contracts::ErrorResponse;
use qervon_application::ApplicationError;
use qervon_domain::DomainError;

#[derive(Debug)]
pub struct ApiError {
    pub status: StatusCode,
    pub detail: String,
}

impl ApiError {
    pub fn unprocessable(detail: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            detail: detail.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorResponse {
            status: self.status.as_u16(),
            title: self
                .status
                .canonical_reason()
                .unwrap_or("Error")
                .to_string(),
            detail: self.detail,
        });
        (self.status, body).into_response()
    }
}

impl From<DomainError> for ApiError {
    fn from(error: DomainError) -> Self {
        match error {
            DomainError::Validation(detail) => Self {
                status: StatusCode::BAD_REQUEST,
                detail,
            },
            DomainError::InvalidTransition(detail) => Self {
                status: StatusCode::CONFLICT,
                detail,
            },
            DomainError::NotFound(detail) => Self {
                status: StatusCode::NOT_FOUND,
                detail,
            },
            DomainError::AlreadyExists(detail) => Self {
                status: StatusCode::CONFLICT,
                detail,
            },
        }
    }
}

impl From<ApplicationError> for ApiError {
    fn from(error: ApplicationError) -> Self {
        match error {
            ApplicationError::Domain(domain) => Self::from(domain),
            ApplicationError::NotFound => Self {
                status: StatusCode::NOT_FOUND,
                detail: "resource not found".to_string(),
            },
            ApplicationError::Conflict(detail) => Self {
                status: StatusCode::CONFLICT,
                detail,
            },
        }
    }
}
