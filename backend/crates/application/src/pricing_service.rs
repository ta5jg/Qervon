// =============================================================================
// File:           backend/crates/application/src/pricing_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Distance-based delivery pricing use cases: reading a tenant's pricing
//   (falling back to a documented default), quoting a fare for a
//   pickup/dropoff pair, and updating a tenant's pricing configuration.
//   The customer-facing order-creation flow always calls `quote_fare`
//   server-side — a client-supplied fare is never trusted.
//
// Specification:
//   QAS-000002, QAS-000005, QAS-000006, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::Utc;
use qervon_domain::{DeliveryPricing, DeliveryPricingRepository, Location, TenantId};

use crate::error::ApplicationError;

#[derive(Debug, Clone, PartialEq)]
pub struct FareQuote {
    pub fare_minor: i64,
    pub currency: String,
    pub distance_km: f64,
}

pub struct PricingService<R>
where
    R: DeliveryPricingRepository,
{
    pricing: R,
}

impl<R> PricingService<R>
where
    R: DeliveryPricingRepository,
{
    pub fn new(pricing: R) -> Self {
        Self { pricing }
    }

    /// The tenant's configured pricing, or the documented default if they
    /// have never configured one. Never persists the default — it is only
    /// materialized into a row once a tenant explicitly saves one via
    /// `update_pricing`.
    pub async fn get_pricing(
        &self,
        tenant_id: TenantId,
    ) -> Result<DeliveryPricing, ApplicationError> {
        match self.pricing.find_by_tenant(tenant_id).await? {
            Some(pricing) => Ok(pricing),
            None => Ok(DeliveryPricing::default_for_tenant(tenant_id, Utc::now())),
        }
    }

    pub async fn quote_fare(
        &self,
        tenant_id: TenantId,
        pickup: &Location,
        dropoff: &Location,
    ) -> Result<FareQuote, ApplicationError> {
        let pricing = self.get_pricing(tenant_id).await?;
        let distance_km = pickup.distance_km(dropoff);
        Ok(FareQuote {
            fare_minor: pricing.quote_fare_minor(distance_km),
            currency: pricing.currency,
            distance_km,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_pricing(
        &self,
        tenant_id: TenantId,
        base_fare_minor: i64,
        per_km_rate_minor: i64,
        minimum_fare_minor: i64,
        currency: String,
    ) -> Result<DeliveryPricing, ApplicationError> {
        let pricing = DeliveryPricing::new(
            tenant_id,
            base_fare_minor,
            per_km_rate_minor,
            minimum_fare_minor,
            currency,
            Utc::now(),
        )?;
        self.pricing.upsert(&pricing).await?;
        Ok(pricing)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qervon_domain::{
        DEFAULT_BASE_FARE_MINOR, DEFAULT_CURRENCY, DEFAULT_MINIMUM_FARE_MINOR,
        DEFAULT_PER_KM_RATE_MINOR,
    };
    use qervon_infrastructure::memory::InMemoryStore;

    #[tokio::test]
    async fn an_unconfigured_tenant_gets_the_documented_defaults() {
        let store = InMemoryStore::new();
        let service = PricingService::new(store.delivery_pricing_repository());
        let pricing = service
            .get_pricing(TenantId::new())
            .await
            .expect("default pricing");
        assert_eq!(pricing.base_fare_minor, DEFAULT_BASE_FARE_MINOR);
        assert_eq!(pricing.per_km_rate_minor, DEFAULT_PER_KM_RATE_MINOR);
        assert_eq!(pricing.minimum_fare_minor, DEFAULT_MINIMUM_FARE_MINOR);
        assert_eq!(pricing.currency, DEFAULT_CURRENCY);
    }

    #[tokio::test]
    async fn quoting_a_fare_uses_the_haversine_distance_between_pickup_and_dropoff() {
        let store = InMemoryStore::new();
        let service = PricingService::new(store.delivery_pricing_repository());
        let tenant_id = TenantId::new();
        let pickup = Location::new(41.0, 29.0).unwrap();
        let dropoff = Location::new(41.0, 29.0).unwrap();

        let quote = service
            .quote_fare(tenant_id, &pickup, &dropoff)
            .await
            .expect("quote");
        assert_eq!(quote.distance_km, 0.0);
        assert_eq!(quote.fare_minor, DEFAULT_MINIMUM_FARE_MINOR);
        assert_eq!(quote.currency, DEFAULT_CURRENCY);
    }

    #[tokio::test]
    async fn updating_pricing_is_reflected_in_subsequent_quotes() {
        let store = InMemoryStore::new();
        let service = PricingService::new(store.delivery_pricing_repository());
        let tenant_id = TenantId::new();

        service
            .update_pricing(tenant_id, 2_000, 500, 3_000, "USD".into())
            .await
            .expect("update pricing");

        let pickup = Location::new(41.0, 29.0).unwrap();
        let dropoff = Location::new(41.0, 29.0).unwrap();
        let quote = service
            .quote_fare(tenant_id, &pickup, &dropoff)
            .await
            .expect("quote");
        assert_eq!(quote.currency, "USD");
        assert_eq!(quote.fare_minor, 3_000);
    }

    #[tokio::test]
    async fn updating_pricing_with_invalid_values_is_rejected() {
        let store = InMemoryStore::new();
        let service = PricingService::new(store.delivery_pricing_repository());
        let err = service
            .update_pricing(TenantId::new(), -1, 500, 3_000, "USD".into())
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::Domain(_)));
    }

    #[tokio::test]
    async fn pricing_is_isolated_per_tenant() {
        let store = InMemoryStore::new();
        let service = PricingService::new(store.delivery_pricing_repository());
        let tenant_a = TenantId::new();
        let tenant_b = TenantId::new();

        service
            .update_pricing(tenant_a, 2_000, 500, 3_000, "USD".into())
            .await
            .expect("update pricing for tenant a");

        let pricing_b = service.get_pricing(tenant_b).await.expect("default for b");
        assert_eq!(pricing_b.base_fare_minor, DEFAULT_BASE_FARE_MINOR);
    }
}
