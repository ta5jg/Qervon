// =============================================================================
// File:           backend/crates/domain/src/cold_chain.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   Cold-Chain Sensor Telemetry & Temperature Violation Alarm Engine.
//
// Specification:
//   QAS-000003, QES-000002.
// =============================================================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColdChainTelemetry {
    pub order_id: uuid::Uuid,
    pub sensor_id: String,
    pub temperature_celsius: f64,
    pub humidity_percent: f64,
    pub min_allowed_temp: f64,
    pub max_allowed_temp: f64,
    pub is_violation: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ColdChainTelemetry {
    pub fn new(
        order_id: uuid::Uuid,
        sensor_id: impl Into<String>,
        temp: f64,
        humidity: f64,
        min_allowed: f64,
        max_allowed: f64,
    ) -> Self {
        let is_violation = temp < min_allowed || temp > max_allowed;
        Self {
            order_id,
            sensor_id: sensor_id.into(),
            temperature_celsius: temp,
            humidity_percent: humidity,
            min_allowed_temp: min_allowed,
            max_allowed_temp: max_allowed,
            is_violation,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_cold_chain_temperature_breach() {
        let order_id = uuid::Uuid::now_v7();
        
        // Medical Vaccine requirement: +2°C to +8°C
        let normal = ColdChainTelemetry::new(order_id, "SENS-101", 5.0, 45.0, 2.0, 8.0);
        assert!(!normal.is_violation);

        let breach = ColdChainTelemetry::new(order_id, "SENS-101", 12.5, 45.0, 2.0, 8.0);
        assert!(breach.is_violation);
    }
}
