// =============================================================================
// File:           backend/crates/application/src/ai_dispatcher.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   AI Dispatcher score computation based on distance, vehicle type efficiency, and ETA heuristics.
//
// Specification:
//   QAS-000002, QAS-000003, QLS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use qervon_domain::{Courier, Location, VehicleType};

#[derive(Debug, Clone, PartialEq)]
pub struct DispatchScore {
    pub courier_id: uuid::Uuid,
    pub score: f64,
    pub distance_km: f64,
    pub estimated_eta_minutes: f64,
}

pub struct AiDispatcher;

impl AiDispatcher {
    /// Calculate dispatch score for a courier given pickup location.
    /// Lower score is better.
    pub fn calculate_score(courier: &Courier, pickup_location: &Location) -> Option<DispatchScore> {
        let current_location = courier.current_location?;
        let distance_km = current_location.distance_km(pickup_location);

        // Average speed heuristic by vehicle type (km/h)
        let avg_speed_kmh = match courier.vehicle {
            VehicleType::Bicycle => 15.0,
            VehicleType::Motorcycle => 35.0,
            VehicleType::Car => 25.0, // Traffic penalty
        };

        let estimated_eta_minutes = (distance_km / avg_speed_kmh) * 60.0;
        
        // Vehicle priority weight multiplier (lower = preferred for short distances)
        let vehicle_weight = match courier.vehicle {
            VehicleType::Motorcycle => 1.0,
            VehicleType::Car => 1.2,
            VehicleType::Bicycle => if distance_km < 3.0 { 0.9 } else { 2.0 },
        };

        let score = (estimated_eta_minutes * 0.7 + distance_km * 0.3) * vehicle_weight;

        Some(DispatchScore {
            courier_id: courier.id,
            score,
            distance_km,
            estimated_eta_minutes,
        })
    }

    /// Rank candidates by AI dispatch score in ascending order (best candidate first).
    pub fn rank_candidates(
        couriers: &[Courier],
        pickup_location: &Location,
    ) -> Vec<DispatchScore> {
        let mut scores: Vec<DispatchScore> = couriers
            .iter()
            .filter_map(|c| Self::calculate_score(c, pickup_location))
            .collect();

        scores.sort_by(|a, b| a.score.total_cmp(&b.score));
        scores
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn ranks_motorcycle_closer_than_car() {
        let loc_pickup = Location::new(41.0, 29.0).unwrap();
        let loc_courier = Location::new(41.02, 29.02).unwrap();

        let mut c1 = Courier::create(uuid::Uuid::now_v7(), "Courier 1", VehicleType::Car, Utc::now()).unwrap();
        c1.set_location(loc_courier);

        let mut c2 = Courier::create(uuid::Uuid::now_v7(), "Courier 2", VehicleType::Motorcycle, Utc::now()).unwrap();
        c2.set_location(loc_courier);

        let candidates = vec![c1, c2.clone()];
        let ranked = AiDispatcher::rank_candidates(&candidates, &loc_pickup);

        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].courier_id, c2.id); // Motorcycle has higher speed & better weight
    }
}
