// =============================================================================
// File:           backend/crates/application/src/currency_exchange.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   Multi-Currency Exchange Rate Engine for International Logistics Settlements.
//
// Specification:
//   QAS-000006, QES-000006.
// =============================================================================
// STATUS: wired -- currency conversion service is exposed via api-gateway endpoints in LOS campaign rollout.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyExchangeEngine;

impl CurrencyExchangeEngine {
    /// Convert minor currency amounts between TRY, USD, EUR, GBP
    pub fn convert_amount(
        amount_minor: i64,
        from_currency: &str,
        to_currency: &str,
    ) -> Result<i64, String> {
        if from_currency == to_currency {
            return Ok(amount_minor);
        }

        // Base exchange rates relative to TRY (e.g. 1 USD = 33.0 TRY, 1 EUR = 36.0 TRY, 1 GBP = 42.0 TRY)
        let rate_to_try = match from_currency {
            "TRY" => 1.0,
            "USD" => 33.0,
            "EUR" => 36.0,
            "GBP" => 42.0,
            _ => return Err(format!("Unsupported source currency: {}", from_currency)),
        };

        let rate_from_try = match to_currency {
            "TRY" => 1.0,
            "USD" => 1.0 / 33.0,
            "EUR" => 1.0 / 36.0,
            "GBP" => 1.0 / 42.0,
            _ => return Err(format!("Unsupported target currency: {}", to_currency)),
        };

        let amount_in_try = (amount_minor as f64) * rate_to_try;
        let converted_amount = (amount_in_try * rate_from_try) as i64;

        Ok(converted_amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_usd_to_try_and_eur() {
        // $100.00 USD (10000 minor) -> TRY = ₺3,300.00 (330000 minor)
        let in_try = CurrencyExchangeEngine::convert_amount(10000, "USD", "TRY").unwrap();
        assert_eq!(in_try, 330000);

        // Same currency returns identical amount
        assert_eq!(
            CurrencyExchangeEngine::convert_amount(5000, "EUR", "EUR").unwrap(),
            5000
        );
    }
}
