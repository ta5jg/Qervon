// =============================================================================
// File:           backend/crates/application/src/fleet_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Fleet use cases: vehicle registration, assignment, maintenance lifecycle.
//
// Specification:
//   QLS-000006, QAS-000002, QAS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{NaiveDate, Utc};
use qervon_domain::{Vehicle, VehicleId, VehicleRepository, VehicleType};
use uuid::Uuid;

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct RegisterVehicleInput {
    pub plate_number: String,
    pub vehicle_type: VehicleType,
    pub insurance_expiry: Option<NaiveDate>,
}

pub struct FleetService<R>
where
    R: VehicleRepository,
{
    vehicles: R,
}

impl<R> FleetService<R>
where
    R: VehicleRepository,
{
    pub fn new(vehicles: R) -> Self {
        Self { vehicles }
    }

    pub async fn register(
        &self,
        input: RegisterVehicleInput,
    ) -> Result<Vehicle, ApplicationError> {
        // Check for duplicate plate.
        if self
            .vehicles
            .find_by_plate(&input.plate_number)
            .await?
            .is_some()
        {
            return Err(ApplicationError::Conflict(format!(
                "a vehicle with plate '{}' already exists",
                input.plate_number
            )));
        }
        let vehicle = Vehicle::register(
            VehicleId::new(),
            input.plate_number,
            input.vehicle_type,
            input.insurance_expiry,
            Utc::now(),
        )?;
        self.vehicles.create(&vehicle).await?;
        Ok(vehicle)
    }

    pub async fn get(&self, id: VehicleId) -> Result<Vehicle, ApplicationError> {
        self.vehicles
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    pub async fn list_active(&self) -> Result<Vec<Vehicle>, ApplicationError> {
        Ok(self.vehicles.list_active().await?)
    }

    pub async fn assign_courier(
        &self,
        vehicle_id: VehicleId,
        courier_id: Uuid,
    ) -> Result<Vehicle, ApplicationError> {
        let mut vehicle = self.get(vehicle_id).await?;
        vehicle.assign_courier(courier_id)?;
        self.vehicles.update(&vehicle).await?;
        Ok(vehicle)
    }

    pub async fn send_to_maintenance(
        &self,
        vehicle_id: VehicleId,
    ) -> Result<Vehicle, ApplicationError> {
        let mut vehicle = self.get(vehicle_id).await?;
        vehicle.send_to_maintenance()?;
        self.vehicles.update(&vehicle).await?;
        Ok(vehicle)
    }

    pub async fn activate(
        &self,
        vehicle_id: VehicleId,
    ) -> Result<Vehicle, ApplicationError> {
        let mut vehicle = self.get(vehicle_id).await?;
        vehicle.activate()?;
        self.vehicles.update(&vehicle).await?;
        Ok(vehicle)
    }

    pub async fn decommission(
        &self,
        vehicle_id: VehicleId,
    ) -> Result<Vehicle, ApplicationError> {
        let mut vehicle = self.get(vehicle_id).await?;
        vehicle.decommission()?;
        self.vehicles.update(&vehicle).await?;
        Ok(vehicle)
    }
}
