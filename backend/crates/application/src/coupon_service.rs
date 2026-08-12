// =============================================================================
// File:           backend/crates/application/src/coupon_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Coupon use cases: tenant-scoped creation/listing and applying a coupon
//   code to an order fare (validating and atomically recording redemption).
//
// Specification:
//   QAS-000005, QAS-000006, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use qervon_domain::{Coupon, CouponRepository, TenantId};

use crate::error::ApplicationError;
use crate::promo_coupon::{PromoCoupon, PromoCouponEngine};

pub struct CouponService<R>
where
    R: CouponRepository,
{
    coupons: R,
}

impl<R> CouponService<R>
where
    R: CouponRepository,
{
    pub fn new(coupons: R) -> Self {
        Self { coupons }
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn create_coupon(
        &self,
        tenant_id: TenantId,
        code: String,
        discount_percent: f64,
        max_discount_minor: i64,
        valid_until: DateTime<Utc>,
        usage_limit: u32,
    ) -> Result<Coupon, ApplicationError> {
        let normalized_code = code.trim().to_uppercase();
        if self
            .coupons
            .find_by_code(tenant_id, &normalized_code)
            .await?
            .is_some()
        {
            return Err(ApplicationError::Conflict(
                "a coupon with this code already exists for this tenant".into(),
            ));
        }
        let coupon = Coupon::create(
            tenant_id,
            code,
            discount_percent,
            max_discount_minor,
            valid_until,
            usage_limit,
            Utc::now(),
        )?;
        self.coupons.create(&coupon).await?;
        Ok(coupon)
    }

    pub async fn list_for_tenant(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<Coupon>, ApplicationError> {
        Ok(self.coupons.list_for_tenant(tenant_id).await?)
    }

    /// Validates and redeems a coupon against a fare, returning the
    /// discounted fare (never below zero) and the updated coupon. The
    /// redemption count is persisted before returning, so a caller that
    /// subsequently fails to create the order does not get a free retry of
    /// this coupon — a known, documented simplification (see
    /// BACKEND_BACKLOG.md) since there is no cross-aggregate saga here.
    pub async fn apply_to_fare(
        &self,
        tenant_id: TenantId,
        code: &str,
        fare_minor: i64,
    ) -> Result<(i64, Coupon), ApplicationError> {
        let normalized_code = code.trim().to_uppercase();
        let mut coupon = self
            .coupons
            .find_by_code(tenant_id, &normalized_code)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        coupon.redeem(Utc::now())?;
        let discounted_fare_minor = PromoCouponEngine::apply_coupon(
            &PromoCoupon {
                code: coupon.code.clone(),
                discount_percent: coupon.discount_percent,
                max_discount_minor: coupon.max_discount_minor,
                is_active: coupon.is_active,
            },
            fare_minor,
        )
        .map_err(ApplicationError::Conflict)?;
        self.coupons.update(&coupon).await?;
        Ok((discounted_fare_minor, coupon))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use qervon_infrastructure::memory::InMemoryStore;

    #[tokio::test]
    async fn creating_a_duplicate_code_is_rejected() {
        let store = InMemoryStore::new();
        let service = CouponService::new(store.coupon_repository());
        let tenant_id = TenantId::new();
        let valid_until = Utc::now() + Duration::days(30);

        service
            .create_coupon(tenant_id, "QERVON20".into(), 20.0, 1000, valid_until, 100)
            .await
            .expect("first coupon");
        let err = service
            .create_coupon(tenant_id, "qervon20".into(), 10.0, 500, valid_until, 50)
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::Conflict(_)));
    }

    #[tokio::test]
    async fn applying_a_coupon_discounts_the_fare_and_tracks_usage() {
        let store = InMemoryStore::new();
        let service = CouponService::new(store.coupon_repository());
        let tenant_id = TenantId::new();
        let valid_until = Utc::now() + Duration::days(30);
        service
            .create_coupon(tenant_id, "QERVON20".into(), 20.0, 1000, valid_until, 1)
            .await
            .expect("create coupon");

        let (discounted, coupon) = service
            .apply_to_fare(tenant_id, "qervon20", 10_000)
            .await
            .expect("apply coupon");
        assert_eq!(discounted, 9_000);
        assert_eq!(coupon.used_count, 1);

        // Usage limit of 1 is now exhausted.
        let err = service
            .apply_to_fare(tenant_id, "QERVON20", 10_000)
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::Domain(_)));
    }

    #[tokio::test]
    async fn a_coupon_from_another_tenant_is_not_found() {
        let store = InMemoryStore::new();
        let service = CouponService::new(store.coupon_repository());
        let tenant_id = TenantId::new();
        let other_tenant_id = TenantId::new();
        service
            .create_coupon(
                tenant_id,
                "ONLYMINE".into(),
                10.0,
                500,
                Utc::now() + Duration::days(30),
                10,
            )
            .await
            .expect("create coupon");

        let err = service
            .apply_to_fare(other_tenant_id, "ONLYMINE", 10_000)
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::NotFound));
    }
}
