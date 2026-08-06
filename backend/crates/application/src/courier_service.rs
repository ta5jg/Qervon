// =============================================================================
// File:           backend/crates/application/src/courier_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Courier lifecycle use cases: registration, location, availability.
//
// Specification:
//   QAS-000002, QLS-000004, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::Utc;
use qervon_domain::{Courier, CourierRepository, Location, VehicleType};
use uuid::Uuid;

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct RegisterCourierInput {
    pub id: Uuid,
    pub name: String,
    pub vehicle: VehicleType,
}

pub struct CourierService<R>
where
    R: CourierRepository,
{
    couriers: R,
}

impl<R> CourierService<R>
where
    R: CourierRepository,
{
    pub fn new(couriers: R) -> Self {
        Self { couriers }
    }

    pub async fn register(&self, input: RegisterCourierInput) -> Result<Courier, ApplicationError> {
        let courier = Courier::create(input.id, input.name, input.vehicle, Utc::now())?;
        self.couriers.create(&courier).await?;
        Ok(courier)
    }

    pub async fn get(&self, id: Uuid) -> Result<Courier, ApplicationError> {
        self.couriers
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    pub async fn list_available(&self) -> Result<Vec<Courier>, ApplicationError> {
        Ok(self.couriers.list_available().await?)
    }

    pub async fn update_location(
        &self,
        id: Uuid,
        location: Location,
    ) -> Result<Courier, ApplicationError> {
        let mut courier = self.get(id).await?;
        courier.set_location(location);
        self.couriers.update(&courier).await?;
        Ok(courier)
    }
}
