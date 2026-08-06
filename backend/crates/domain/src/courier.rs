// =============================================================================
// File:           backend/crates/domain/src/courier.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Courier aggregate with availability and location invariants.
//
// Specification:
//   QAS-000002, QAS-000003, QLS-000004, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;
use crate::location::Location;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VehicleType {
    Bicycle,
    Motorcycle,
    Car,
}

impl VehicleType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bicycle => "bicycle",
            Self::Motorcycle => "motorcycle",
            Self::Car => "car",
        }
    }
}

impl std::str::FromStr for VehicleType {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "bicycle" => Ok(Self::Bicycle),
            "motorcycle" => Ok(Self::Motorcycle),
            "car" => Ok(Self::Car),
            other => Err(DomainError::validation(format!(
                "unknown vehicle type: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for VehicleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CourierStatus {
    Available,
    Busy,
    Offline,
}

impl CourierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Busy => "busy",
            Self::Offline => "offline",
        }
    }

    pub fn can_accept_work(&self) -> bool {
        matches!(self, Self::Available)
    }
}

impl std::str::FromStr for CourierStatus {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "available" => Ok(Self::Available),
            "busy" => Ok(Self::Busy),
            "offline" => Ok(Self::Offline),
            other => Err(DomainError::validation(format!(
                "unknown courier status: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for CourierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Courier {
    pub id: Uuid,
    pub name: String,
    pub vehicle: VehicleType,
    pub status: CourierStatus,
    pub current_location: Option<Location>,
    pub registered_at: DateTime<Utc>,
}

impl Courier {
    pub fn create(
        id: Uuid,
        name: impl Into<String>,
        vehicle: VehicleType,
        registered_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        let name = name.into();
        if id.is_nil() {
            return Err(DomainError::validation("courier id is required"));
        }
        if name.trim().is_empty() {
            return Err(DomainError::validation("courier name is required"));
        }
        Ok(Self {
            id,
            name,
            vehicle,
            status: CourierStatus::Available,
            current_location: None,
            registered_at,
        })
    }

    pub fn set_location(&mut self, location: Location) {
        self.current_location = Some(location);
    }

    pub fn go_busy(&mut self) -> Result<(), DomainError> {
        if !self.status.can_accept_work() {
            return Err(DomainError::invalid_transition(format!(
                "courier {} is not available (status {:?})",
                self.id, self.status
            )));
        }
        self.status = CourierStatus::Busy;
        Ok(())
    }

    pub fn go_available(&mut self) -> Result<(), DomainError> {
        if !matches!(self.status, CourierStatus::Busy) {
            return Err(DomainError::invalid_transition(format!(
                "courier {} is not busy (status {:?})",
                self.id, self.status
            )));
        }
        self.status = CourierStatus::Available;
        Ok(())
    }

    pub fn go_offline(&mut self) -> Result<(), DomainError> {
        if !matches!(self.status, CourierStatus::Available) {
            return Err(DomainError::invalid_transition(format!(
                "courier {} cannot go offline while {:?}",
                self.id, self.status
            )));
        }
        self.status = CourierStatus::Offline;
        Ok(())
    }

    pub fn distance_to(&self, target: &Location) -> Result<f64, DomainError> {
        self.current_location
            .map(|origin| origin.distance_km(target))
            .ok_or_else(|| DomainError::NotFound("courier has no location".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_courier() -> Courier {
        Courier::create(
            Uuid::now_v7(),
            "Ayşe Kurye",
            VehicleType::Motorcycle,
            Utc::now(),
        )
        .expect("valid courier")
    }

    #[test]
    fn registered_courier_is_available() {
        assert_eq!(sample_courier().status, CourierStatus::Available);
    }

    #[test]
    fn goes_busy_then_available() {
        let mut courier = sample_courier();
        courier.go_busy().expect("go busy");
        assert_eq!(courier.status, CourierStatus::Busy);
        courier.go_available().expect("go available");
        assert_eq!(courier.status, CourierStatus::Available);
    }

    #[test]
    fn busy_courier_cannot_accept_more_work() {
        let mut courier = sample_courier();
        courier.go_busy().expect("go busy");
        let err = courier.go_busy().unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn only_available_courier_can_go_offline() {
        let mut courier = sample_courier();
        courier.go_offline().expect("offline");
        let err = courier.go_busy().unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn rejects_blank_name() {
        assert!(Courier::create(Uuid::now_v7(), "  ", VehicleType::Car, Utc::now()).is_err());
    }
}
