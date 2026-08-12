// =============================================================================
// File:           backend/crates/domain/src/coupon.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Promo coupon domain model: a tenant-scoped, persisted, usage-limited
//   discount code. The actual discount arithmetic lives in
//   `qervon_application::promo_coupon::PromoCouponEngine`; this entity only
//   owns the coupon's identity, validity window, and redemption bookkeeping.
//
// Specification:
//   QAS-000005, QAS-000006, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::tenant::TenantId;
use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Coupon {
    pub id: Uuid,
    pub tenant_id: TenantId,
    pub code: String,
    pub discount_percent: f64,
    pub max_discount_minor: i64,
    pub valid_until: DateTime<Utc>,
    pub usage_limit: u32,
    pub used_count: u32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl Coupon {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        tenant_id: TenantId,
        code: impl Into<String>,
        discount_percent: f64,
        max_discount_minor: i64,
        valid_until: DateTime<Utc>,
        usage_limit: u32,
        now: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let code = code.into().trim().to_uppercase();
        if code.is_empty() {
            return Err(DomainError::validation("coupon code is required"));
        }
        if !(0.0..=100.0).contains(&discount_percent) {
            return Err(DomainError::validation(
                "discount percent must be between 0 and 100",
            ));
        }
        if max_discount_minor < 0 {
            return Err(DomainError::validation(
                "max discount amount cannot be negative",
            ));
        }
        if usage_limit == 0 {
            return Err(DomainError::validation("usage limit must be positive"));
        }
        Ok(Self {
            id: Uuid::now_v7(),
            tenant_id,
            code,
            discount_percent,
            max_discount_minor,
            valid_until,
            usage_limit,
            used_count: 0,
            is_active: true,
            created_at: now,
        })
    }

    pub fn is_redeemable(&self, now: DateTime<Utc>) -> bool {
        self.is_active && now <= self.valid_until && self.used_count < self.usage_limit
    }

    /// Marks one redemption. Callers must persist the resulting state; this
    /// only mutates the in-memory aggregate.
    pub fn redeem(&mut self, now: DateTime<Utc>) -> Result<(), DomainError> {
        if !self.is_redeemable(now) {
            return Err(DomainError::invalid_transition(
                "coupon is expired, inactive, or has reached its usage limit",
            ));
        }
        self.used_count += 1;
        Ok(())
    }

    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn tenant() -> TenantId {
        TenantId::new()
    }

    #[test]
    fn rejects_invalid_discount_percent() {
        let now = Utc::now();
        let err = Coupon::create(
            tenant(),
            "QERVON20",
            150.0,
            1000,
            now + Duration::days(1),
            10,
            now,
        )
        .unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn rejects_zero_usage_limit() {
        let now = Utc::now();
        let err = Coupon::create(
            tenant(),
            "QERVON20",
            10.0,
            1000,
            now + Duration::days(1),
            0,
            now,
        )
        .unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
    }

    #[test]
    fn normalizes_code_to_uppercase() {
        let now = Utc::now();
        let coupon = Coupon::create(
            tenant(),
            " qervon20 ",
            10.0,
            1000,
            now + Duration::days(1),
            10,
            now,
        )
        .expect("valid coupon");
        assert_eq!(coupon.code, "QERVON20");
    }

    #[test]
    fn redeeming_increments_usage_until_limit_reached() {
        let now = Utc::now();
        let mut coupon = Coupon::create(
            tenant(),
            "LIMITED",
            10.0,
            1000,
            now + Duration::days(1),
            2,
            now,
        )
        .expect("valid coupon");
        coupon.redeem(now).expect("first redemption");
        coupon.redeem(now).expect("second redemption");
        let err = coupon.redeem(now).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn expired_coupon_cannot_be_redeemed() {
        let now = Utc::now();
        let mut coupon = Coupon::create(
            tenant(),
            "EXPIRED",
            10.0,
            1000,
            now - Duration::days(1),
            10,
            now - Duration::days(2),
        )
        .expect("valid coupon");
        let err = coupon.redeem(now).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn deactivated_coupon_cannot_be_redeemed() {
        let now = Utc::now();
        let mut coupon = Coupon::create(
            tenant(),
            "OFF",
            10.0,
            1000,
            now + Duration::days(1),
            10,
            now,
        )
        .expect("valid coupon");
        coupon.deactivate();
        assert!(coupon.redeem(now).is_err());
    }
}
