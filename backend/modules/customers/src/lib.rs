// =============================================================================
// File:           backend/modules/customers/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Customers domain module: public boundary over customer profile use cases.
//
// Specification:
//   QLS-000005, QAS-000002, QAS-000004, QAS-000005.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use qervon_application::CustomerService;
use qervon_domain::{CustomerId, CustomerProfile, CustomerRepository, Location, UserId};
use uuid::Uuid;

pub struct CustomersModule<R>
where
    R: CustomerRepository,
{
    service: CustomerService<R>,
}

impl<R> CustomersModule<R>
where
    R: CustomerRepository,
{
    pub fn new(customers: R) -> Self {
        Self {
            service: CustomerService::new(customers),
        }
    }

    pub async fn create_profile(
        &self,
        user_id: UserId,
    ) -> Result<CustomerProfile, qervon_application::ApplicationError> {
        self.service.create_profile(user_id).await
    }

    pub async fn get_profile(
        &self,
        id: CustomerId,
    ) -> Result<CustomerProfile, qervon_application::ApplicationError> {
        self.service.get(id).await
    }

    pub async fn get_profile_by_user(
        &self,
        user_id: UserId,
    ) -> Result<CustomerProfile, qervon_application::ApplicationError> {
        self.service.get_by_user(user_id).await
    }

    pub async fn add_address(
        &self,
        customer_id: CustomerId,
        label: String,
        location: Location,
        full_address: String,
    ) -> Result<CustomerProfile, qervon_application::ApplicationError> {
        self.service
            .add_address(customer_id, label, location, full_address)
            .await
    }

    pub async fn remove_address(
        &self,
        customer_id: CustomerId,
        address_id: Uuid,
    ) -> Result<CustomerProfile, qervon_application::ApplicationError> {
        self.service.remove_address(customer_id, address_id).await
    }

    pub async fn add_loyalty_points(
        &self,
        customer_id: CustomerId,
        points: u64,
    ) -> Result<CustomerProfile, qervon_application::ApplicationError> {
        self.service.add_loyalty_points(customer_id, points).await
    }
}
