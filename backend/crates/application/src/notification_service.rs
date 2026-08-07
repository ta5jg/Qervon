// =============================================================================
// File:           backend/crates/application/src/notification_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Notification use cases: create, send, and manage notification lifecycle.
//
// Specification:
//   QLS-000010, QAS-000002, QAS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::Utc;
use qervon_domain::{
    Notification, NotificationChannel, NotificationId, NotificationRepository,
};
use uuid::Uuid;

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct SendNotificationInput {
    pub recipient_id: Uuid,
    pub channel: NotificationChannel,
    pub title: String,
    pub body: String,
}

pub struct NotificationService<R>
where
    R: NotificationRepository,
{
    notifications: R,
}

impl<R> NotificationService<R>
where
    R: NotificationRepository,
{
    pub fn new(notifications: R) -> Self {
        Self { notifications }
    }

    /// Queue a new notification for delivery.
    pub async fn send(
        &self,
        input: SendNotificationInput,
    ) -> Result<Notification, ApplicationError> {
        let notification = Notification::create(
            NotificationId::new(),
            input.recipient_id,
            input.channel,
            input.title,
            input.body,
            Utc::now(),
        )?;
        self.notifications.create(&notification).await?;
        Ok(notification)
    }

    pub async fn get(
        &self,
        id: NotificationId,
    ) -> Result<Notification, ApplicationError> {
        self.notifications
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    /// List all notifications for a given recipient.
    pub async fn list_for_recipient(
        &self,
        recipient_id: Uuid,
    ) -> Result<Vec<Notification>, ApplicationError> {
        Ok(self.notifications.list_for_recipient(recipient_id).await?)
    }

    /// Mark a queued notification as successfully delivered.
    pub async fn mark_sent(
        &self,
        id: NotificationId,
    ) -> Result<Notification, ApplicationError> {
        let mut notification = self.get(id).await?;
        notification.mark_sent(Utc::now())?;
        self.notifications.update(&notification).await?;
        Ok(notification)
    }

    /// Mark a queued notification as failed to deliver.
    pub async fn mark_failed(
        &self,
        id: NotificationId,
    ) -> Result<Notification, ApplicationError> {
        let mut notification = self.get(id).await?;
        notification.mark_failed()?;
        self.notifications.update(&notification).await?;
        Ok(notification)
    }

    /// Mark a delivered notification as read by the recipient.
    pub async fn mark_read(
        &self,
        id: NotificationId,
    ) -> Result<Notification, ApplicationError> {
        let mut notification = self.get(id).await?;
        notification.mark_read()?;
        self.notifications.update(&notification).await?;
        Ok(notification)
    }
}
