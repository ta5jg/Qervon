// =============================================================================
// File:           backend/crates/domain/src/location.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Geographic location value object (WGS84 latitude/longitude).
//
// Specification:
//   QAS-000002, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use serde::{Deserialize, Serialize};

use crate::error::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
}

impl Location {
    pub fn new(latitude: f64, longitude: f64) -> Result<Self, DomainError> {
        if !latitude.is_finite() || !longitude.is_finite() {
            return Err(DomainError::validation(
                "latitude and longitude must be finite numbers",
            ));
        }
        if !(-90.0..=90.0).contains(&latitude) {
            return Err(DomainError::validation("latitude must be within [-90, 90]"));
        }
        if !(-180.0..=180.0).contains(&longitude) {
            return Err(DomainError::validation(
                "longitude must be within [-180, 180]",
            ));
        }
        Ok(Self {
            latitude,
            longitude,
        })
    }

    /// Great-circle distance in kilometres using the haversine formula.
    pub fn distance_km(&self, other: &Location) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6_371.0;

        let lat1 = self.latitude.to_radians();
        let lat2 = other.latitude.to_radians();
        let d_lat = (other.latitude - self.latitude).to_radians();
        let d_lon = (other.longitude - self.longitude).to_radians();

        let a = (d_lat / 2.0).sin().powi(2) + lat1.cos() * lat2.cos() * (d_lon / 2.0).sin().powi(2);
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        EARTH_RADIUS_KM * c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_valid_coordinates() {
        let location = Location::new(41.0082, 28.9784).expect("valid Istanbul coords");
        assert_eq!(location.latitude, 41.0082);
    }

    #[test]
    fn rejects_out_of_range_latitude() {
        assert!(Location::new(91.0, 0.0).is_err());
        assert!(Location::new(-91.0, 0.0).is_err());
    }

    #[test]
    fn rejects_non_finite_values() {
        assert!(Location::new(f64::NAN, 0.0).is_err());
        assert!(Location::new(0.0, f64::INFINITY).is_err());
    }

    #[test]
    fn zero_distance_for_identical_locations() {
        let a = Location::new(10.0, 20.0).unwrap();
        assert_eq!(a.distance_km(&a), 0.0);
    }

    #[test]
    fn approximate_distance_between_known_points() {
        let istanbul = Location::new(41.0082, 28.9784).unwrap();
        let ankara = Location::new(39.9334, 32.8597).unwrap();
        let distance = istanbul.distance_km(&ankara);
        assert!((distance - 351.0).abs() < 10.0, "distance was {distance}");
    }
}
