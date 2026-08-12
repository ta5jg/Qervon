// =============================================================================
// File:           backend/crates/domain/src/delivery_pricing.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Per-tenant, distance-based delivery pricing configuration. The customer
//   app never supplies its own fare — the server always computes it from
//   this configuration (or a sane default when a tenant has not configured
//   one yet), so a client can never manipulate the price it is charged.
//
// Specification:
//   QAS-000002, QAS-000005, QLS-000002, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::error::DomainError;
use crate::money::Money;
use crate::tenant::TenantId;

/// Sane out-of-the-box defaults so a tenant that has never configured
/// pricing still gets a real, working fare rather than an error. Expressed
/// in minor units of `DEFAULT_CURRENCY`.
pub const DEFAULT_BASE_FARE_MINOR: i64 = 1_000;
pub const DEFAULT_PER_KM_RATE_MINOR: i64 = 250;
pub const DEFAULT_MINIMUM_FARE_MINOR: i64 = 1_500;
pub const DEFAULT_CURRENCY: &str = "TRY";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeliveryPricing {
    pub tenant_id: TenantId,
    pub base_fare_minor: i64,
    pub per_km_rate_minor: i64,
    pub minimum_fare_minor: i64,
    pub currency: String,
    pub updated_at: DateTime<Utc>,
}

impl DeliveryPricing {
    pub fn new(
        tenant_id: TenantId,
        base_fare_minor: i64,
        per_km_rate_minor: i64,
        minimum_fare_minor: i64,
        currency: impl Into<String>,
        updated_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if base_fare_minor < 0 || per_km_rate_minor < 0 || minimum_fare_minor < 0 {
            return Err(DomainError::validation(
                "pricing amounts cannot be negative",
            ));
        }
        // `Money::new` already validates the ISO 4217 currency shape; reuse
        // it purely for that check (a zero amount is always valid there).
        let currency = Money::new(0, currency)?.currency;
        Ok(Self {
            tenant_id,
            base_fare_minor,
            per_km_rate_minor,
            minimum_fare_minor,
            currency,
            updated_at,
        })
    }

    /// The pricing every tenant gets until they explicitly configure their
    /// own via `PUT /v1/pricing`.
    pub fn default_for_tenant(tenant_id: TenantId, now: DateTime<Utc>) -> Self {
        Self {
            tenant_id,
            base_fare_minor: DEFAULT_BASE_FARE_MINOR,
            per_km_rate_minor: DEFAULT_PER_KM_RATE_MINOR,
            minimum_fare_minor: DEFAULT_MINIMUM_FARE_MINOR,
            currency: DEFAULT_CURRENCY.to_string(),
            updated_at: now,
        }
    }

    /// `max(minimum, base + per_km * distance)`, rounded to the nearest
    /// minor unit.
    pub fn quote_fare_minor(&self, distance_km: f64) -> i64 {
        let distance_km = distance_km.max(0.0);
        let variable = (self.per_km_rate_minor as f64) * distance_km;
        let computed = self.base_fare_minor + variable.round() as i64;
        computed.max(self.minimum_fare_minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pricing_quotes_base_plus_distance() {
        let pricing = DeliveryPricing::default_for_tenant(TenantId::new(), Utc::now());
        // 2km: base 1000 + 2*250 = 1500, tied with the minimum.
        assert_eq!(pricing.quote_fare_minor(2.0), 1_500);
        // 10km: base 1000 + 10*250 = 3500, above the minimum.
        assert_eq!(pricing.quote_fare_minor(10.0), 3_500);
    }

    #[test]
    fn short_distance_is_floored_at_the_minimum_fare() {
        let pricing = DeliveryPricing::default_for_tenant(TenantId::new(), Utc::now());
        assert_eq!(pricing.quote_fare_minor(0.0), DEFAULT_MINIMUM_FARE_MINOR);
        assert_eq!(pricing.quote_fare_minor(0.1), DEFAULT_MINIMUM_FARE_MINOR);
    }

    #[test]
    fn negative_distance_is_treated_as_zero() {
        let pricing = DeliveryPricing::default_for_tenant(TenantId::new(), Utc::now());
        assert_eq!(
            pricing.quote_fare_minor(-5.0),
            pricing.quote_fare_minor(0.0)
        );
    }

    #[test]
    fn rejects_negative_amounts() {
        let now = Utc::now();
        assert!(DeliveryPricing::new(TenantId::new(), -1, 250, 1_500, "TRY", now).is_err());
        assert!(DeliveryPricing::new(TenantId::new(), 1_000, -1, 1_500, "TRY", now).is_err());
        assert!(DeliveryPricing::new(TenantId::new(), 1_000, 250, -1, "TRY", now).is_err());
    }

    #[test]
    fn rejects_malformed_currency() {
        let now = Utc::now();
        assert!(DeliveryPricing::new(TenantId::new(), 1_000, 250, 1_500, "try", now).is_err());
    }

    #[test]
    fn custom_pricing_overrides_defaults() {
        let now = Utc::now();
        let pricing = DeliveryPricing::new(TenantId::new(), 2_000, 500, 3_000, "USD", now)
            .expect("valid pricing");
        assert_eq!(pricing.quote_fare_minor(4.0), 4_000);
        assert_eq!(pricing.currency, "USD");
    }
}
