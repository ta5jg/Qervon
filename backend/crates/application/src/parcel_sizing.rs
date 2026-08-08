// =============================================================================
// File:           backend/crates/application/src/parcel_sizing.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   Volumetric Weight (Desi) & Parcel Vehicle Compatibility Engine.
//
// Specification:
//   QAS-000003, QES-000006.
// =============================================================================

use qervon_domain::VehicleType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParcelDimensions {
    pub width_cm: f64,
    pub height_cm: f64,
    pub length_cm: f64,
    pub actual_weight_kg: f64,
}

pub struct ParcelSizingEngine;

impl ParcelSizingEngine {
    /// Calculate volumetric weight (Desi = Width * Height * Length / 5000)
    pub fn calculate_desi(dims: &ParcelDimensions) -> f64 {
        (dims.width_cm * dims.height_cm * dims.length_cm) / 5000.0
    }

    /// Determine billable weight (Max of actual weight vs volumetric desi weight)
    pub fn get_billable_weight(dims: &ParcelDimensions) -> f64 {
        let desi = Self::calculate_desi(dims);
        desi.max(dims.actual_weight_kg)
    }

    /// Check if parcel fits vehicle type (e.g. Motorcycle max 15 Desi, Bicycle max 5 Desi)
    pub fn is_compatible_with_vehicle(dims: &ParcelDimensions, vehicle: VehicleType) -> bool {
        let billable = Self::get_billable_weight(dims);
        match vehicle {
            VehicleType::Bicycle => billable <= 5.0,
            VehicleType::Motorcycle => billable <= 20.0,
            VehicleType::Car => billable <= 150.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_desi_and_vehicle_compatibility() {
        let box_small = ParcelDimensions {
            width_cm: 20.0,
            height_cm: 20.0,
            length_cm: 20.0, // 8000 / 5000 = 1.6 Desi
            actual_weight_kg: 1.0,
        };

        let desi = ParcelSizingEngine::calculate_desi(&box_small);
        assert_eq!(desi, 1.6);
        assert!(ParcelSizingEngine::is_compatible_with_vehicle(&box_small, VehicleType::Motorcycle));
        assert!(ParcelSizingEngine::is_compatible_with_vehicle(&box_small, VehicleType::Bicycle));

        let box_large = ParcelDimensions {
            width_cm: 60.0,
            height_cm: 50.0,
            length_cm: 50.0, // 150,000 / 5000 = 30 Desi
            actual_weight_kg: 10.0,
        };

        assert!(!ParcelSizingEngine::is_compatible_with_vehicle(&box_large, VehicleType::Motorcycle));
        assert!(ParcelSizingEngine::is_compatible_with_vehicle(&box_large, VehicleType::Car));
    }
}
