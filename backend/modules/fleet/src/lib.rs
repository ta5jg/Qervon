// =============================================================================
// File:           backend/modules/fleet/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Fleet domain module: public boundary over vehicle use cases.
//
// Specification:
//   QLS-000006, QAS-000002, QAS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use qervon_application::{FleetService, RegisterVehicleInput};
use qervon_domain::{Vehicle, VehicleId, VehicleRepository};
use uuid::Uuid;

pub struct FleetModule<R>
where
    R: VehicleRepository,
{
    service: FleetService<R>,
}

impl<R> FleetModule<R>
where
    R: VehicleRepository,
{
    pub fn new(vehicles: R) -> Self {
        Self {
            service: FleetService::new(vehicles),
        }
    }

    pub async fn register_vehicle(
        &self,
        input: RegisterVehicleInput,
    ) -> Result<Vehicle, qervon_application::ApplicationError> {
        self.service.register(input).await
    }

    pub async fn get_vehicle(
        &self,
        id: VehicleId,
    ) -> Result<Vehicle, qervon_application::ApplicationError> {
        self.service.get(id).await
    }

    pub async fn list_active_vehicles(
        &self,
    ) -> Result<Vec<Vehicle>, qervon_application::ApplicationError> {
        self.service.list_active().await
    }

    pub async fn assign_courier(
        &self,
        vehicle_id: VehicleId,
        courier_id: Uuid,
    ) -> Result<Vehicle, qervon_application::ApplicationError> {
        self.service.assign_courier(vehicle_id, courier_id).await
    }

    pub async fn send_to_maintenance(
        &self,
        vehicle_id: VehicleId,
    ) -> Result<Vehicle, qervon_application::ApplicationError> {
        self.service.send_to_maintenance(vehicle_id).await
    }

    pub async fn activate(
        &self,
        vehicle_id: VehicleId,
    ) -> Result<Vehicle, qervon_application::ApplicationError> {
        self.service.activate(vehicle_id).await
    }

    pub async fn decommission(
        &self,
        vehicle_id: VehicleId,
    ) -> Result<Vehicle, qervon_application::ApplicationError> {
        self.service.decommission(vehicle_id).await
    }
}
