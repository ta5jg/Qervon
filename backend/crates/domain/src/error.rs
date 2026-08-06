// =============================================================================
// File:           backend/crates/domain/src/error.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Canonical domain error type shared across Qervon domain modules.
//
// Specification:
//   QAS-000002, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("invalid state transition: {0}")]
    InvalidTransition(String),
    #[error("resource not found: {0}")]
    NotFound(String),
    #[error("resource already exists: {0}")]
    AlreadyExists(String),
}

impl DomainError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub fn invalid_transition(message: impl Into<String>) -> Self {
        Self::InvalidTransition(message.into())
    }
}
