// =============================================================================
// File:           backend/modules/couriers/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Couriers domain module: public boundary over courier use cases.
//
// Specification:
//   QAS-000001 through QAS-000006, QLS-000004, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use qervon_application::{CourierService, RegisterCourierInput};
use qervon_domain::{Courier, CourierRepository, Location};
use uuid::Uuid;

pub struct CouriersModule<R>
where
    R: CourierRepository,
{
    service: CourierService<R>,
}

impl<R> CouriersModule<R>
where
    R: CourierRepository,
{
    pub fn new(couriers: R) -> Self {
        Self {
            service: CourierService::new(couriers),
        }
    }

    pub async fn register_courier(
        &self,
        input: RegisterCourierInput,
    ) -> Result<Courier, qervon_application::ApplicationError> {
        self.service.register(input).await
    }

    pub async fn get_courier(
        &self,
        id: Uuid,
    ) -> Result<Courier, qervon_application::ApplicationError> {
        self.service.get(id).await
    }

    pub async fn list_available_couriers(
        &self,
    ) -> Result<Vec<Courier>, qervon_application::ApplicationError> {
        self.service.list_available().await
    }

    pub async fn list_all_couriers(
        &self,
    ) -> Result<Vec<Courier>, qervon_application::ApplicationError> {
        self.service.list_all().await
    }

    pub async fn update_courier_location(
        &self,
        id: Uuid,
        location: Location,
    ) -> Result<Courier, qervon_application::ApplicationError> {
        self.service.update_location(id, location).await
    }

    pub async fn set_courier_online_status(
        &self,
        id: Uuid,
        online: bool,
    ) -> Result<Courier, qervon_application::ApplicationError> {
        self.service.set_online_status(id, online).await
    }
}
