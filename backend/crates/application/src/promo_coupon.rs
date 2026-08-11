// =============================================================================
// File:           backend/crates/application/src/promo_coupon.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   Promo Coupon, Discount Calculation & Loyalty Hub.
//
// Specification:
//   QAS-000005, QES-000006.
// =============================================================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromoCoupon {
    pub code: String, // e.g. "QERVON20"
    pub discount_percent: f64,
    pub max_discount_minor: i64,
    pub is_active: bool,
}

pub struct PromoCouponEngine;

impl PromoCouponEngine {
    /// Apply discount coupon to order fare
    pub fn apply_coupon(coupon: &PromoCoupon, original_fare_minor: i64) -> Result<i64, String> {
        if !coupon.is_active {
            return Err("Coupon is expired or inactive".into());
        }

        let calculated_discount =
            ((original_fare_minor as f64) * (coupon.discount_percent / 100.0)) as i64;
        let final_discount = calculated_discount.min(coupon.max_discount_minor);
        let final_fare = (original_fare_minor - final_discount).max(0);

        Ok(final_fare)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn applies_promo_coupon_with_discount_cap() {
        let coupon = PromoCoupon {
            code: "QERVON20".into(),
            discount_percent: 20.0,   // 20% discount
            max_discount_minor: 1000, // Max ₺10.00 discount
            is_active: true,
        };

        // ₺100.00 fare -> %20 = ₺20.00 -> Capped at ₺10.00 -> Final ₺90.00
        let discounted_fare = PromoCouponEngine::apply_coupon(&coupon, 10000).unwrap();
        assert_eq!(discounted_fare, 9000);
    }
}
