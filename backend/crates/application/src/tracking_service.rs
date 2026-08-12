// =============================================================================
// File:           backend/crates/application/src/tracking_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Tracking use cases: start/end sessions, record location points.
//
// Specification:
//   QLS-000007, QAS-000002, QAS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use qervon_domain::{Location, TrackingPoint, TrackingRepository, TrackingSession};
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

use crate::ai_dispatcher::AiDispatcher;
use crate::error::ApplicationError;

pub struct TrackingService<R>
where
    R: TrackingRepository,
{
    tracking: R,
    /// Last known (location, timestamp) per courier, used only to detect
    /// physically impossible speed jumps between consecutive updates. This
    /// is an in-memory hint, not a source of truth: it is intentionally not
    /// persisted and resets on restart, which is acceptable since fraud
    /// detection only needs the immediately preceding sample.
    recent_locations: RwLock<HashMap<Uuid, (Location, DateTime<Utc>)>>,
}

impl<R> TrackingService<R>
where
    R: TrackingRepository,
{
    pub fn new(tracking: R) -> Self {
        Self {
            tracking,
            recent_locations: RwLock::new(HashMap::new()),
        }
    }

    /// Start a new tracking session for a courier going online.
    pub async fn start_session(
        &self,
        courier_id: Uuid,
    ) -> Result<TrackingSession, ApplicationError> {
        // Ensure no duplicate active sessions.
        if let Some(existing) = self
            .tracking
            .find_active_session_for_courier(courier_id)
            .await?
        {
            return Err(ApplicationError::Conflict(format!(
                "courier {} already has an active tracking session: {}",
                courier_id, existing.id
            )));
        }
        let session = TrackingSession::start(courier_id, Utc::now())?;
        self.tracking.create_session(&session).await?;
        Ok(session)
    }

    /// End the active tracking session for a courier going offline.
    pub async fn end_session(&self, courier_id: Uuid) -> Result<TrackingSession, ApplicationError> {
        let mut session = self
            .tracking
            .find_active_session_for_courier(courier_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        session.end(Utc::now())?;
        self.tracking.update_session(&session).await?;
        Ok(session)
    }

    /// Record a single location ping from a courier.
    ///
    /// Runs the AI Fraud Guard against the courier's immediately preceding
    /// sample before persisting: an implied speed above the physical
    /// threshold flags the point as suspicious, but the location is still
    /// recorded (flag-and-accept), never rejected outright.
    pub async fn record_location(
        &self,
        courier_id: Uuid,
        location: Location,
        speed_kmh: Option<f64>,
        battery_pct: Option<u8>,
    ) -> Result<TrackingPoint, ApplicationError> {
        let now = Utc::now();
        let mut point = TrackingPoint::new(courier_id, location, speed_kmh, battery_pct, now)?;

        if let Some((previous_location, previous_at)) = self.previous_location(courier_id) {
            // Nanosecond precision matters here: two updates can legitimately
            // arrive sub-millisecond apart (e.g. in tests or batched relays),
            // and millisecond truncation would zero out the elapsed time and
            // silently skip the fraud check via `detect_gps_fraud`'s
            // `elapsed_seconds <= 0.0` guard.
            let elapsed_seconds =
                (now - previous_at).num_nanoseconds().unwrap_or(0) as f64 / 1_000_000_000.0;
            let (is_fraudulent, risk_score) =
                AiDispatcher::detect_gps_fraud(&previous_location, &location, elapsed_seconds);
            if is_fraudulent {
                point.flag_fraud_risk(risk_score);
                tracing::warn!(
                    courier_id = %courier_id,
                    risk_score,
                    elapsed_seconds,
                    "AI Fraud Guard flagged an implausible GPS jump"
                );
            }
        }
        self.remember_location(courier_id, location, now);

        self.tracking.record_point(&point).await?;
        Ok(point)
    }

    fn previous_location(&self, courier_id: Uuid) -> Option<(Location, DateTime<Utc>)> {
        self.recent_locations
            .read()
            .ok()
            .and_then(|cache| cache.get(&courier_id).copied())
    }

    fn remember_location(&self, courier_id: Uuid, location: Location, at: DateTime<Utc>) {
        if let Ok(mut cache) = self.recent_locations.write() {
            cache.insert(courier_id, (location, at));
        }
    }
}
