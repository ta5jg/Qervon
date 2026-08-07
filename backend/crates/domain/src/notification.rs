// =============================================================================
// File:           backend/crates/domain/src/notification.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Notification domain: multi-channel notification lifecycle.
//
// Specification:
//   QLS-000010, QAS-000002, QAS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;

// ---------------------------------------------------------------------------
// NotificationId
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NotificationId(pub Uuid);

impl NotificationId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for NotificationId {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// NotificationChannel
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationChannel {
    Push,
    Sms,
    Email,
    WhatsApp,
}

impl NotificationChannel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Push => "push",
            Self::Sms => "sms",
            Self::Email => "email",
            Self::WhatsApp => "whatsapp",
        }
    }
}

impl std::str::FromStr for NotificationChannel {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "push" => Ok(Self::Push),
            "sms" => Ok(Self::Sms),
            "email" => Ok(Self::Email),
            "whatsapp" => Ok(Self::WhatsApp),
            other => Err(DomainError::validation(format!(
                "unknown notification channel: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for NotificationChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// NotificationStatus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationStatus {
    Queued,
    Sent,
    Failed,
    Read,
}

impl NotificationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Sent => "sent",
            Self::Failed => "failed",
            Self::Read => "read",
        }
    }
}

impl std::str::FromStr for NotificationStatus {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "queued" => Ok(Self::Queued),
            "sent" => Ok(Self::Sent),
            "failed" => Ok(Self::Failed),
            "read" => Ok(Self::Read),
            other => Err(DomainError::validation(format!(
                "unknown notification status: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for NotificationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Notification entity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Notification {
    pub id: NotificationId,
    pub recipient_id: Uuid,
    pub channel: NotificationChannel,
    pub title: String,
    pub body: String,
    pub status: NotificationStatus,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
}

impl Notification {
    pub fn create(
        id: NotificationId,
        recipient_id: Uuid,
        channel: NotificationChannel,
        title: impl Into<String>,
        body: impl Into<String>,
        now: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if recipient_id.is_nil() {
            return Err(DomainError::validation("recipient id is required"));
        }
        let title = title.into();
        let body = body.into();
        if title.trim().is_empty() {
            return Err(DomainError::validation("notification title is required"));
        }
        if body.trim().is_empty() {
            return Err(DomainError::validation("notification body is required"));
        }
        Ok(Self {
            id,
            recipient_id,
            channel,
            title,
            body,
            status: NotificationStatus::Queued,
            created_at: now,
            sent_at: None,
        })
    }

    pub fn mark_sent(&mut self, now: DateTime<Utc>) -> Result<(), DomainError> {
        if self.status != NotificationStatus::Queued {
            return Err(DomainError::invalid_transition(format!(
                "cannot mark {} notification as sent",
                self.status
            )));
        }
        self.status = NotificationStatus::Sent;
        self.sent_at = Some(now);
        Ok(())
    }

    pub fn mark_failed(&mut self) -> Result<(), DomainError> {
        if self.status != NotificationStatus::Queued {
            return Err(DomainError::invalid_transition(format!(
                "cannot mark {} notification as failed",
                self.status
            )));
        }
        self.status = NotificationStatus::Failed;
        Ok(())
    }

    pub fn mark_read(&mut self) -> Result<(), DomainError> {
        if self.status != NotificationStatus::Sent {
            return Err(DomainError::invalid_transition(format!(
                "cannot mark {} notification as read",
                self.status
            )));
        }
        self.status = NotificationStatus::Read;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_notification() -> Notification {
        Notification::create(
            NotificationId::new(),
            Uuid::now_v7(),
            NotificationChannel::Push,
            "Sipariş Güncelleme",
            "Siparişiniz teslim edildi.",
            Utc::now(),
        )
        .expect("valid notification")
    }

    #[test]
    fn notification_starts_queued() {
        let n = sample_notification();
        assert_eq!(n.status, NotificationStatus::Queued);
        assert!(n.sent_at.is_none());
    }

    #[test]
    fn full_lifecycle_queued_sent_read() {
        let mut n = sample_notification();
        n.mark_sent(Utc::now()).expect("sent");
        assert_eq!(n.status, NotificationStatus::Sent);
        assert!(n.sent_at.is_some());

        n.mark_read().expect("read");
        assert_eq!(n.status, NotificationStatus::Read);
    }

    #[test]
    fn queued_can_fail() {
        let mut n = sample_notification();
        n.mark_failed().expect("failed");
        assert_eq!(n.status, NotificationStatus::Failed);
    }

    #[test]
    fn sent_notification_cannot_fail() {
        let mut n = sample_notification();
        n.mark_sent(Utc::now()).expect("sent");
        let err = n.mark_failed().unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn queued_notification_cannot_be_read() {
        let mut n = sample_notification();
        let err = n.mark_read().unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn rejects_blank_title() {
        let result = Notification::create(
            NotificationId::new(),
            Uuid::now_v7(),
            NotificationChannel::Sms,
            "  ",
            "body text",
            Utc::now(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn rejects_nil_recipient() {
        let result = Notification::create(
            NotificationId::new(),
            Uuid::nil(),
            NotificationChannel::Email,
            "Title",
            "Body",
            Utc::now(),
        );
        assert!(result.is_err());
    }
}
