// =============================================================================
// File:           backend/crates/domain/src/tracking.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Tracking domain: courier location points and tracking sessions.
//
// Specification:
//   QLS-000007, QAS-000002, QAS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;
use crate::location::Location;

// ---------------------------------------------------------------------------
// TrackingPoint — a single location sample from a courier
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackingPoint {
    pub id: Uuid,
    pub courier_id: Uuid,
    pub location: Location,
    pub speed_kmh: Option<f64>,
    pub battery_pct: Option<u8>,
    pub recorded_at: DateTime<Utc>,
    /// Set by the AI Fraud Guard when this point implies a physically
    /// impossible speed from the courier's previous point. The point is
    /// still recorded (flag-and-accept), never rejected outright.
    pub fraud_flagged: bool,
    /// Normalized 0.0–1.0 anomaly score backing `fraud_flagged`.
    pub fraud_risk_score: f64,
}

impl TrackingPoint {
    pub fn new(
        courier_id: Uuid,
        location: Location,
        speed_kmh: Option<f64>,
        battery_pct: Option<u8>,
        recorded_at: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if courier_id.is_nil() {
            return Err(DomainError::validation("courier id is required"));
        }
        if let Some(speed) = speed_kmh {
            if speed < 0.0 || !speed.is_finite() {
                return Err(DomainError::validation(
                    "speed must be a non-negative finite number",
                ));
            }
        }
        if let Some(battery) = battery_pct {
            if battery > 100 {
                return Err(DomainError::validation("battery percentage must be 0–100"));
            }
        }
        Ok(Self {
            id: Uuid::now_v7(),
            courier_id,
            location,
            speed_kmh,
            battery_pct,
            recorded_at,
            fraud_flagged: false,
            fraud_risk_score: 0.0,
        })
    }

    /// Marks this point as suspicious per the AI Fraud Guard's speed-anomaly
    /// check. Recording still proceeds; this only annotates the sample.
    pub fn flag_fraud_risk(&mut self, risk_score: f64) {
        self.fraud_flagged = true;
        self.fraud_risk_score = risk_score;
    }
}

// ---------------------------------------------------------------------------
// TrackingSession — groups consecutive tracking points for a courier shift
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackingSessionStatus {
    Active,
    Ended,
}

impl TrackingSessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Ended => "ended",
        }
    }
}

impl std::str::FromStr for TrackingSessionStatus {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "active" => Ok(Self::Active),
            "ended" => Ok(Self::Ended),
            other => Err(DomainError::validation(format!(
                "unknown tracking session status: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for TrackingSessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackingSession {
    pub id: Uuid,
    pub courier_id: Uuid,
    pub status: TrackingSessionStatus,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
}

impl TrackingSession {
    pub fn start(courier_id: Uuid, now: DateTime<Utc>) -> Result<Self, DomainError> {
        if courier_id.is_nil() {
            return Err(DomainError::validation("courier id is required"));
        }
        Ok(Self {
            id: Uuid::now_v7(),
            courier_id,
            status: TrackingSessionStatus::Active,
            started_at: now,
            ended_at: None,
        })
    }

    pub fn end(&mut self, now: DateTime<Utc>) -> Result<(), DomainError> {
        if self.status != TrackingSessionStatus::Active {
            return Err(DomainError::invalid_transition(
                "can only end an active tracking session",
            ));
        }
        if now < self.started_at {
            return Err(DomainError::validation(
                "end time cannot precede start time",
            ));
        }
        self.status = TrackingSessionStatus::Ended;
        self.ended_at = Some(now);
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.status == TrackingSessionStatus::Active
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::Location;

    fn istanbul() -> Location {
        Location::new(41.0082, 28.9784).unwrap()
    }

    #[test]
    fn tracking_point_requires_courier_id() {
        let result = TrackingPoint::new(Uuid::nil(), istanbul(), None, None, Utc::now());
        assert!(result.is_err());
    }

    #[test]
    fn tracking_point_rejects_negative_speed() {
        let result = TrackingPoint::new(Uuid::now_v7(), istanbul(), Some(-5.0), None, Utc::now());
        assert!(result.is_err());
    }

    #[test]
    fn tracking_point_rejects_battery_over_100() {
        let result = TrackingPoint::new(Uuid::now_v7(), istanbul(), None, Some(101), Utc::now());
        assert!(result.is_err());
    }

    #[test]
    fn valid_tracking_point_stores_data() {
        let point =
            TrackingPoint::new(Uuid::now_v7(), istanbul(), Some(30.0), Some(85), Utc::now())
                .expect("valid point");
        assert_eq!(point.speed_kmh, Some(30.0));
        assert_eq!(point.battery_pct, Some(85));
        assert!(!point.fraud_flagged);
        assert_eq!(point.fraud_risk_score, 0.0);
    }

    #[test]
    fn flagging_fraud_risk_marks_the_point_without_rejecting_it() {
        let mut point = TrackingPoint::new(Uuid::now_v7(), istanbul(), None, None, Utc::now())
            .expect("valid point");
        point.flag_fraud_risk(0.95);
        assert!(point.fraud_flagged);
        assert_eq!(point.fraud_risk_score, 0.95);
    }

    #[test]
    fn session_starts_active() {
        let session = TrackingSession::start(Uuid::now_v7(), Utc::now()).expect("valid session");
        assert!(session.is_active());
        assert!(session.ended_at.is_none());
    }

    #[test]
    fn session_can_be_ended() {
        let mut session =
            TrackingSession::start(Uuid::now_v7(), Utc::now()).expect("valid session");
        session.end(Utc::now()).expect("end session");
        assert!(!session.is_active());
        assert!(session.ended_at.is_some());
    }

    #[test]
    fn ended_session_cannot_be_ended_again() {
        let mut session =
            TrackingSession::start(Uuid::now_v7(), Utc::now()).expect("valid session");
        session.end(Utc::now()).expect("end once");
        let err = session.end(Utc::now()).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }
}
