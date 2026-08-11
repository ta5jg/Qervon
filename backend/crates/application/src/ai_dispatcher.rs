// =============================================================================
// File:           backend/crates/application/src/ai_dispatcher.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.2.0
//
// Description:
//   AI Dispatcher score computation, Dynamic ETA Engine (weather/traffic factor),
//   and AI Fraud Detection (Mock Location & Speed Anomaly Guard).
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
    pub fraud_risk_score: f64,
}

#[derive(Debug, Clone, Copy)]
pub enum WeatherCondition {
    Clear,
    Rainy,
    Snowy,
}

#[derive(Debug, Clone, Copy)]
pub struct TrafficContext {
    pub congestion_multiplier: f64, // 1.0 = Normal, 1.5 = Heavy traffic
    pub weather: WeatherCondition,
}

pub struct AiDispatcher;

impl AiDispatcher {
    /// Dynamic ETA Engine considering distance, vehicle speed, traffic, and weather.
    pub fn calculate_dynamic_eta(
        distance_km: f64,
        vehicle: VehicleType,
        context: Option<TrafficContext>,
    ) -> f64 {
        let base_speed_kmh = match vehicle {
            VehicleType::Bicycle => 15.0,
            VehicleType::Motorcycle => 35.0,
            VehicleType::Car => 25.0,
        };

        let traffic_factor = context.map_or(1.0, |c| c.congestion_multiplier);
        let weather_factor = match context.map(|c| c.weather) {
            Some(WeatherCondition::Rainy) => 1.25,
            Some(WeatherCondition::Snowy) => 1.60,
            _ => 1.0,
        };

        let effective_speed = base_speed_kmh / (traffic_factor * weather_factor);
        (distance_km / effective_speed) * 60.0
    }

    /// AI Fraud Detection: Check for impossible speed/teleportation anomaly
    pub fn detect_gps_fraud(
        old_location: &Location,
        new_location: &Location,
        elapsed_seconds: f64,
    ) -> (bool, f64) {
        if elapsed_seconds <= 0.0 {
            return (false, 0.0);
        }
        let dist_km = old_location.distance_km(new_location);
        let speed_kmh = (dist_km / elapsed_seconds) * 3600.0;

        // Anomaly threshold: Speed > 160 km/h for city courier movement is flagged as Fraud Risk
        let is_fraudulent = speed_kmh > 160.0;
        let risk_score = (speed_kmh / 200.0).min(1.0);

        (is_fraudulent, risk_score)
    }

    /// Calculate dispatch score for a courier given pickup location and traffic context.
    pub fn calculate_score(
        courier: &Courier,
        pickup_location: &Location,
        context: Option<TrafficContext>,
    ) -> Option<DispatchScore> {
        let current_location = courier.current_location?;
        let distance_km = current_location.distance_km(pickup_location);

        let estimated_eta_minutes =
            Self::calculate_dynamic_eta(distance_km, courier.vehicle, context);

        let vehicle_weight = match courier.vehicle {
            VehicleType::Motorcycle => 1.0,
            VehicleType::Car => 1.2,
            VehicleType::Bicycle => {
                if distance_km < 3.0 {
                    0.9
                } else {
                    2.0
                }
            }
        };

        let score = (estimated_eta_minutes * 0.7 + distance_km * 0.3) * vehicle_weight;

        Some(DispatchScore {
            courier_id: courier.id,
            score,
            distance_km,
            estimated_eta_minutes,
            fraud_risk_score: 0.0,
        })
    }

    /// Rank candidates by AI dispatch score in ascending order (best candidate first).
    pub fn rank_candidates(couriers: &[Courier], pickup_location: &Location) -> Vec<DispatchScore> {
        let mut scores: Vec<DispatchScore> = couriers
            .iter()
            .filter_map(|c| Self::calculate_score(c, pickup_location, None))
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

        let mut c1 = Courier::create(
            uuid::Uuid::now_v7(),
            "Courier 1",
            VehicleType::Car,
            Utc::now(),
        )
        .unwrap();
        c1.set_location(loc_courier);

        let mut c2 = Courier::create(
            uuid::Uuid::now_v7(),
            "Courier 2",
            VehicleType::Motorcycle,
            Utc::now(),
        )
        .unwrap();
        c2.set_location(loc_courier);

        let candidates = vec![c1, c2.clone()];
        let ranked = AiDispatcher::rank_candidates(&candidates, &loc_pickup);

        assert_eq!(ranked.len(), 2);
        assert_eq!(ranked[0].courier_id, c2.id);
    }

    #[test]
    fn detects_impossible_speed_fraud() {
        let loc_a = Location::new(41.0, 28.9).unwrap();
        let loc_b = Location::new(41.5, 29.5).unwrap(); // ~70 km distance

        let (is_fraud, risk) = AiDispatcher::detect_gps_fraud(&loc_a, &loc_b, 10.0); // 70km in 10 seconds!
        assert!(is_fraud);
        assert!(risk > 0.8);
    }
}
