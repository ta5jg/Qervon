// =============================================================================
// File:           backend/crates/domain/src/route_history.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   GPS Route History Breadcrumbs & Playback Track Record Domain Model.
//
// Specification:
//   QAS-000003, QES-000002.
// =============================================================================
// STATUS: v2 backlog -- domain model + unit tests only; no repository, migration, or HTTP route yet. See BACKEND_BACKLOG.md.

use crate::Location;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteBreadcrumb {
    pub courier_id: uuid::Uuid,
    pub location: Location,
    pub speed_kmh: f64,
    pub battery_level: u8,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourierPlaybackTrack {
    pub courier_id: uuid::Uuid,
    pub date: String, // YYYY-MM-DD
    pub total_distance_km: f64,
    pub average_speed_kmh: f64,
    pub breadcrumbs: Vec<RouteBreadcrumb>,
}

impl CourierPlaybackTrack {
    pub fn new(courier_id: uuid::Uuid, date: impl Into<String>) -> Self {
        Self {
            courier_id,
            date: date.into(),
            total_distance_km: 0.0,
            average_speed_kmh: 0.0,
            breadcrumbs: Vec::new(),
        }
    }

    pub fn add_breadcrumb(&mut self, breadcrumb: RouteBreadcrumb) {
        if let Some(last) = self.breadcrumbs.last() {
            let dist = last.location.distance_km(&breadcrumb.location);
            self.total_distance_km += dist;
        }

        self.breadcrumbs.push(breadcrumb);

        let total_speed: f64 = self.breadcrumbs.iter().map(|b| b.speed_kmh).sum();
        self.average_speed_kmh = total_speed / self.breadcrumbs.len() as f64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_track_distance_and_avg_speed() {
        let courier_id = uuid::Uuid::now_v7();
        let mut track = CourierPlaybackTrack::new(courier_id, "2026-08-08");

        let b1 = RouteBreadcrumb {
            courier_id,
            location: Location::new(41.06, 28.93).unwrap(),
            speed_kmh: 30.0,
            battery_level: 90,
            timestamp: chrono::Utc::now(),
        };

        let b2 = RouteBreadcrumb {
            courier_id,
            location: Location::new(41.07, 28.94).unwrap(),
            speed_kmh: 40.0,
            battery_level: 88,
            timestamp: chrono::Utc::now(),
        };

        track.add_breadcrumb(b1);
        track.add_breadcrumb(b2);

        assert_eq!(track.breadcrumbs.len(), 2);
        assert!(track.total_distance_km > 0.0);
        assert_eq!(track.average_speed_kmh, 35.0);
    }
}
