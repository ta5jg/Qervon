// =============================================================================
// File:           backend/modules/tracking/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Tracking domain module: public boundary over tracking use cases.
//
// Specification:
//   QLS-000007, QAS-000003, QAS-000006, QAS-000007.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use qervon_application::TrackingService;
use qervon_domain::{Location, TrackingRepository, TrackingSession};
use uuid::Uuid;

pub struct TrackingModule<R>
where
    R: TrackingRepository,
{
    service: TrackingService<R>,
}

impl<R> TrackingModule<R>
where
    R: TrackingRepository,
{
    pub fn new(tracking: R) -> Self {
        Self {
            service: TrackingService::new(tracking),
        }
    }

    pub async fn start_session(
        &self,
        courier_id: Uuid,
    ) -> Result<TrackingSession, qervon_application::ApplicationError> {
        self.service.start_session(courier_id).await
    }

    pub async fn end_session(
        &self,
        courier_id: Uuid,
    ) -> Result<TrackingSession, qervon_application::ApplicationError> {
        self.service.end_session(courier_id).await
    }

    pub async fn record_location(
        &self,
        courier_id: Uuid,
        location: Location,
        speed_kmh: Option<f64>,
        battery_pct: Option<u8>,
    ) -> Result<(), qervon_application::ApplicationError> {
        self.service
            .record_location(courier_id, location, speed_kmh, battery_pct)
            .await
            .map(|_| ())
    }
}
