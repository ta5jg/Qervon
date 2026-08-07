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

use chrono::Utc;
use qervon_domain::{
    Location, TrackingPoint, TrackingRepository, TrackingSession,
};
use uuid::Uuid;

use crate::error::ApplicationError;

pub struct TrackingService<R>
where
    R: TrackingRepository,
{
    tracking: R,
}

impl<R> TrackingService<R>
where
    R: TrackingRepository,
{
    pub fn new(tracking: R) -> Self {
        Self { tracking }
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
    pub async fn end_session(
        &self,
        courier_id: Uuid,
    ) -> Result<TrackingSession, ApplicationError> {
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
    pub async fn record_location(
        &self,
        courier_id: Uuid,
        location: Location,
        speed_kmh: Option<f64>,
        battery_pct: Option<u8>,
    ) -> Result<TrackingPoint, ApplicationError> {
        let point = TrackingPoint::new(courier_id, location, speed_kmh, battery_pct, Utc::now())?;
        self.tracking.record_point(&point).await?;
        Ok(point)
    }
}
