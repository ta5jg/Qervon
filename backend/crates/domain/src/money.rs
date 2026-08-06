// =============================================================================
// File:           backend/crates/domain/src/money.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Money value object in minor units to avoid floating-point arithmetic.
//
// Specification:
//   QAS-000002, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Money {
    /// Amount in minor units (e.g. kuruş for TRY, cent for EUR/USD).
    pub amount_minor: i64,
    /// ISO 4217 currency code.
    pub currency: String,
}

impl Money {
    pub fn new(amount_minor: i64, currency: impl Into<String>) -> Result<Self, DomainError> {
        if amount_minor < 0 {
            return Err(DomainError::validation("amount cannot be negative"));
        }
        let currency = currency.into();
        if !(currency.len() == 3 && currency.chars().all(|c| c.is_ascii_uppercase())) {
            return Err(DomainError::validation(
                "currency must be a 3-letter ISO 4217 code",
            ));
        }
        Ok(Self {
            amount_minor,
            currency,
        })
    }

    pub fn zero() -> Self {
        Self {
            amount_minor: 0,
            currency: "TRY".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_positive_amount_with_iso_currency() {
        let money = Money::new(1_000, "TRY").expect("valid money");
        assert_eq!(money.amount_minor, 1_000);
        assert_eq!(money.currency, "TRY");
    }

    #[test]
    fn rejects_negative_amount() {
        assert!(Money::new(-1, "TRY").is_err());
    }

    #[test]
    fn rejects_malformed_currency() {
        assert!(Money::new(100, "tru").is_err());
        assert!(Money::new(100, "TR").is_err());
    }
}
