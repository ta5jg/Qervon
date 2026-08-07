// =============================================================================
// File:           backend/crates/application/src/customer_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Customer use cases: profile creation, address book, loyalty points.
//
// Specification:
//   QLS-000005, QAS-000002, QAS-000004, QAS-000005, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::Utc;
use qervon_domain::{
    CustomerId, CustomerProfile, CustomerRepository, Location, SavedAddress, UserId,
};
use uuid::Uuid;

use crate::error::ApplicationError;

pub struct CustomerService<R>
where
    R: CustomerRepository,
{
    customers: R,
}

impl<R> CustomerService<R>
where
    R: CustomerRepository,
{
    pub fn new(customers: R) -> Self {
        Self { customers }
    }

    pub async fn create_profile(
        &self,
        user_id: UserId,
    ) -> Result<CustomerProfile, ApplicationError> {
        // Check for existing profile.
        if self.customers.find_by_user(user_id).await?.is_some() {
            return Err(ApplicationError::Conflict(
                "customer profile already exists for this user".into(),
            ));
        }
        let profile = CustomerProfile::create(CustomerId::new(), user_id, Utc::now());
        self.customers.create(&profile).await?;
        Ok(profile)
    }

    pub async fn get(&self, id: CustomerId) -> Result<CustomerProfile, ApplicationError> {
        self.customers
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    pub async fn get_by_user(
        &self,
        user_id: UserId,
    ) -> Result<CustomerProfile, ApplicationError> {
        self.customers
            .find_by_user(user_id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    pub async fn add_address(
        &self,
        customer_id: CustomerId,
        label: String,
        location: Location,
        full_address: String,
    ) -> Result<CustomerProfile, ApplicationError> {
        let mut profile = self.get(customer_id).await?;
        let address = SavedAddress::new(label, location, full_address)?;
        profile.add_address(address);
        self.customers.update(&profile).await?;
        Ok(profile)
    }

    pub async fn remove_address(
        &self,
        customer_id: CustomerId,
        address_id: Uuid,
    ) -> Result<CustomerProfile, ApplicationError> {
        let mut profile = self.get(customer_id).await?;
        profile.remove_address(address_id)?;
        self.customers.update(&profile).await?;
        Ok(profile)
    }

    pub async fn add_loyalty_points(
        &self,
        customer_id: CustomerId,
        points: u64,
    ) -> Result<CustomerProfile, ApplicationError> {
        let mut profile = self.get(customer_id).await?;
        profile.add_loyalty_points(points);
        self.customers.update(&profile).await?;
        Ok(profile)
    }
}
