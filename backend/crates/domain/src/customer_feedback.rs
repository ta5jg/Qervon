// =============================================================================
// File:           backend/crates/domain/src/customer_feedback.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.2.0
//
// Description:
//   Customer Feedback, Ratings & Support Ticket Management Domain Model.
//
// Specification:
//   QAS-000004, QES-000006.
// =============================================================================

use crate::tenant::TenantId;
use crate::DomainError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
}

impl TicketStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in_progress",
            Self::Resolved => "resolved",
            Self::Closed => "closed",
        }
    }
}

impl std::str::FromStr for TicketStatus {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "open" => Ok(Self::Open),
            "in_progress" => Ok(Self::InProgress),
            "resolved" => Ok(Self::Resolved),
            "closed" => Ok(Self::Closed),
            other => Err(DomainError::validation(format!(
                "unknown support ticket status: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for TicketStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerRating {
    pub id: uuid::Uuid,
    pub order_id: uuid::Uuid,
    pub customer_id: uuid::Uuid,
    pub courier_id: uuid::Uuid,
    pub rating_stars: u8, // 1 to 5
    pub comment: Option<String>,
    pub photo_url: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl CustomerRating {
    pub fn new(
        order_id: uuid::Uuid,
        customer_id: uuid::Uuid,
        courier_id: uuid::Uuid,
        rating_stars: u8,
        comment: Option<String>,
    ) -> Result<Self, DomainError> {
        if !(1..=5).contains(&rating_stars) {
            return Err(DomainError::Validation(
                "Rating stars must be between 1 and 5".into(),
            ));
        }

        Ok(Self {
            id: uuid::Uuid::now_v7(),
            order_id,
            customer_id,
            courier_id,
            rating_stars,
            comment,
            photo_url: None,
            created_at: chrono::Utc::now(),
        })
    }
}

/// Support tickets carry an explicit `tenant_id`: unlike a rating (always
/// tied to a delivered order, whose tenant can be derived), a ticket may be
/// raised with no order reference at all, so it needs its own tenant
/// ownership record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportTicket {
    pub id: uuid::Uuid,
    pub tenant_id: TenantId,
    pub customer_id: uuid::Uuid,
    pub order_id: Option<uuid::Uuid>,
    pub subject: String,
    pub message: String,
    pub status: TicketStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SupportTicket {
    pub fn open(
        tenant_id: TenantId,
        customer_id: uuid::Uuid,
        order_id: Option<uuid::Uuid>,
        subject: impl Into<String>,
        message: impl Into<String>,
        now: chrono::DateTime<chrono::Utc>,
    ) -> Result<Self, DomainError> {
        let subject = subject.into();
        let message = message.into();
        if subject.trim().is_empty() {
            return Err(DomainError::validation("subject is required"));
        }
        if message.trim().is_empty() {
            return Err(DomainError::validation("message is required"));
        }
        Ok(Self {
            id: uuid::Uuid::now_v7(),
            tenant_id,
            customer_id,
            order_id,
            subject,
            message,
            status: TicketStatus::Open,
            created_at: now,
        })
    }

    pub fn start_progress(&mut self) -> Result<(), DomainError> {
        if self.status != TicketStatus::Open {
            return Err(DomainError::invalid_transition(format!(
                "cannot start progress on a {} ticket",
                self.status
            )));
        }
        self.status = TicketStatus::InProgress;
        Ok(())
    }

    pub fn resolve(&mut self) -> Result<(), DomainError> {
        if !matches!(self.status, TicketStatus::Open | TicketStatus::InProgress) {
            return Err(DomainError::invalid_transition(format!(
                "cannot resolve a {} ticket",
                self.status
            )));
        }
        self.status = TicketStatus::Resolved;
        Ok(())
    }

    pub fn close(&mut self) -> Result<(), DomainError> {
        if self.status == TicketStatus::Closed {
            return Err(DomainError::invalid_transition("ticket is already closed"));
        }
        self.status = TicketStatus::Closed;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_rating_stars_range() {
        let order_id = uuid::Uuid::now_v7();
        let cust_id = uuid::Uuid::now_v7();
        let courier_id = uuid::Uuid::now_v7();

        assert!(CustomerRating::new(order_id, cust_id, courier_id, 5, None).is_ok());
        assert!(CustomerRating::new(order_id, cust_id, courier_id, 6, None).is_err());
    }

    #[test]
    fn ticket_status_string_round_trip() {
        for variant in [
            TicketStatus::Open,
            TicketStatus::InProgress,
            TicketStatus::Resolved,
            TicketStatus::Closed,
        ] {
            assert_eq!(variant.as_str().parse::<TicketStatus>(), Ok(variant));
        }
    }

    #[test]
    fn rejects_blank_subject_or_message() {
        let tenant_id = TenantId::new();
        let customer_id = uuid::Uuid::now_v7();
        assert!(SupportTicket::open(
            tenant_id,
            customer_id,
            None,
            "  ",
            "message",
            chrono::Utc::now()
        )
        .is_err());
        assert!(SupportTicket::open(
            tenant_id,
            customer_id,
            None,
            "subject",
            "  ",
            chrono::Utc::now()
        )
        .is_err());
    }

    #[test]
    fn ticket_lifecycle_open_progress_resolve() {
        let mut ticket = SupportTicket::open(
            TenantId::new(),
            uuid::Uuid::now_v7(),
            None,
            "Kurye gelmedi",
            "Siparişim 1 saattir bekliyor",
            chrono::Utc::now(),
        )
        .expect("valid ticket");
        assert_eq!(ticket.status, TicketStatus::Open);

        ticket.start_progress().expect("start progress");
        assert_eq!(ticket.status, TicketStatus::InProgress);

        ticket.resolve().expect("resolve");
        assert_eq!(ticket.status, TicketStatus::Resolved);

        ticket.close().expect("close");
        assert_eq!(ticket.status, TicketStatus::Closed);
        let err = ticket.close().unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn cannot_resolve_a_closed_ticket() {
        let mut ticket = SupportTicket::open(
            TenantId::new(),
            uuid::Uuid::now_v7(),
            None,
            "Konu",
            "Mesaj",
            chrono::Utc::now(),
        )
        .expect("valid ticket");
        ticket.close().expect("close");
        let err = ticket.resolve().unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }
}
