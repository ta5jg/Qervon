// =============================================================================
// File:           backend/modules/notifications/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Notifications domain module: public boundary over notification use cases.
//
// Specification:
//   QLS-000010, QAS-000002, QAS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use qervon_application::{NotificationService, SendNotificationInput};
use qervon_domain::{Notification, NotificationId, NotificationRepository};
use uuid::Uuid;

pub struct NotificationsModule<R>
where
    R: NotificationRepository,
{
    service: NotificationService<R>,
}

impl<R> NotificationsModule<R>
where
    R: NotificationRepository,
{
    pub fn new(notifications: R) -> Self {
        Self {
            service: NotificationService::new(notifications),
        }
    }

    pub async fn send(
        &self,
        input: SendNotificationInput,
    ) -> Result<Notification, qervon_application::ApplicationError> {
        self.service.send(input).await
    }

    pub async fn get(
        &self,
        id: NotificationId,
    ) -> Result<Notification, qervon_application::ApplicationError> {
        self.service.get(id).await
    }

    pub async fn list_for_recipient(
        &self,
        recipient_id: Uuid,
    ) -> Result<Vec<Notification>, qervon_application::ApplicationError> {
        self.service.list_for_recipient(recipient_id).await
    }

    pub async fn mark_sent(
        &self,
        id: NotificationId,
    ) -> Result<Notification, qervon_application::ApplicationError> {
        self.service.mark_sent(id).await
    }

    pub async fn mark_read(
        &self,
        id: NotificationId,
    ) -> Result<Notification, qervon_application::ApplicationError> {
        self.service.mark_read(id).await
    }
}
