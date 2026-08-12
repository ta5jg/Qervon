// =============================================================================
// File:           backend/crates/domain/src/fleet.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Fleet domain: vehicle registration, lifecycle, and assignment tracking.
//
// Specification:
//   QLS-000006, QAS-000002, QAS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::courier::VehicleType;
use crate::error::DomainError;

// ---------------------------------------------------------------------------
// VehicleId
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VehicleId(pub Uuid);

impl VehicleId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for VehicleId {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// VehicleStatus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleStatus {
    Active,
    Maintenance,
    Decommissioned,
}

impl VehicleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Maintenance => "maintenance",
            Self::Decommissioned => "decommissioned",
        }
    }
}

impl std::str::FromStr for VehicleStatus {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "active" => Ok(Self::Active),
            "maintenance" => Ok(Self::Maintenance),
            "decommissioned" => Ok(Self::Decommissioned),
            other => Err(DomainError::validation(format!(
                "unknown vehicle status: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for VehicleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Vehicle entity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: VehicleId,
    pub plate_number: String,
    pub vehicle_type: VehicleType,
    pub status: VehicleStatus,
    pub assigned_courier_id: Option<Uuid>,
    pub insurance_expiry: Option<NaiveDate>,
    pub registered_at: DateTime<Utc>,
}

impl Vehicle {
    pub fn register(
        id: VehicleId,
        plate_number: impl Into<String>,
        vehicle_type: VehicleType,
        insurance_expiry: Option<NaiveDate>,
        registered_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let plate_number = plate_number.into();
        if plate_number.trim().is_empty() {
            return Err(DomainError::validation("plate number is required"));
        }
        Ok(Self {
            id,
            plate_number,
            vehicle_type,
            status: VehicleStatus::Active,
            assigned_courier_id: None,
            insurance_expiry,
            registered_at,
        })
    }

    pub fn assign_courier(&mut self, courier_id: Uuid) -> Result<(), DomainError> {
        if self.status != VehicleStatus::Active {
            return Err(DomainError::invalid_transition(format!(
                "cannot assign courier to a {} vehicle",
                self.status
            )));
        }
        if courier_id.is_nil() {
            return Err(DomainError::validation("courier id is required"));
        }
        self.assigned_courier_id = Some(courier_id);
        Ok(())
    }

    pub fn unassign_courier(&mut self) {
        self.assigned_courier_id = None;
    }

    pub fn send_to_maintenance(&mut self) -> Result<(), DomainError> {
        if self.status != VehicleStatus::Active {
            return Err(DomainError::invalid_transition(format!(
                "only active vehicles can be sent to maintenance, current status: {}",
                self.status
            )));
        }
        self.assigned_courier_id = None;
        self.status = VehicleStatus::Maintenance;
        Ok(())
    }

    pub fn activate(&mut self) -> Result<(), DomainError> {
        if self.status != VehicleStatus::Maintenance {
            return Err(DomainError::invalid_transition(format!(
                "only vehicles in maintenance can be activated, current status: {}",
                self.status
            )));
        }
        self.status = VehicleStatus::Active;
        Ok(())
    }

    pub fn decommission(&mut self) -> Result<(), DomainError> {
        if self.status == VehicleStatus::Decommissioned {
            return Err(DomainError::invalid_transition(
                "vehicle is already decommissioned",
            ));
        }
        self.assigned_courier_id = None;
        self.status = VehicleStatus::Decommissioned;
        Ok(())
    }

    pub fn is_insurance_expired(&self, today: NaiveDate) -> bool {
        self.insurance_expiry
            .map(|expiry| expiry < today)
            .unwrap_or(true) // no expiry recorded → treat as expired
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::courier::VehicleType;

    fn sample_vehicle() -> Vehicle {
        Vehicle::register(
            VehicleId::new(),
            "34 ABC 123",
            VehicleType::Motorcycle,
            Some(NaiveDate::from_ymd_opt(2027, 6, 1).unwrap()),
            Utc::now(),
        )
        .expect("valid vehicle")
    }

    #[test]
    fn registered_vehicle_is_active_and_unassigned() {
        let v = sample_vehicle();
        assert_eq!(v.status, VehicleStatus::Active);
        assert!(v.assigned_courier_id.is_none());
    }

    #[test]
    fn rejects_blank_plate() {
        assert!(
            Vehicle::register(VehicleId::new(), "  ", VehicleType::Car, None, Utc::now()).is_err()
        );
    }

    #[test]
    fn assign_courier_to_active_vehicle() {
        let mut v = sample_vehicle();
        v.assign_courier(Uuid::now_v7()).expect("assign");
        assert!(v.assigned_courier_id.is_some());
    }

    #[test]
    fn cannot_assign_courier_to_maintenance_vehicle() {
        let mut v = sample_vehicle();
        v.send_to_maintenance().expect("maintenance");
        let err = v.assign_courier(Uuid::now_v7()).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn maintenance_then_activate_then_decommission() {
        let mut v = sample_vehicle();
        v.send_to_maintenance().expect("maintenance");
        assert_eq!(v.status, VehicleStatus::Maintenance);

        v.activate().expect("activate");
        assert_eq!(v.status, VehicleStatus::Active);

        v.decommission().expect("decommission");
        assert_eq!(v.status, VehicleStatus::Decommissioned);
    }

    #[test]
    fn decommissioned_vehicle_cannot_be_decommissioned_again() {
        let mut v = sample_vehicle();
        v.decommission().expect("first decommission");
        let err = v.decommission().unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn insurance_expiry_check() {
        let v = sample_vehicle();
        let before = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let after = NaiveDate::from_ymd_opt(2028, 1, 1).unwrap();
        assert!(!v.is_insurance_expired(before));
        assert!(v.is_insurance_expired(after));
    }

    #[test]
    fn send_to_maintenance_clears_courier_assignment() {
        let mut v = sample_vehicle();
        v.assign_courier(Uuid::now_v7()).expect("assign");
        v.send_to_maintenance().expect("maintenance");
        assert!(v.assigned_courier_id.is_none());
    }
}
