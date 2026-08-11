// =============================================================================
// File:           backend/crates/domain/src/customer_feedback.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   Customer Feedback, Ratings & Support Ticket Management Domain Model.
//
// Specification:
//   QAS-000004, QES-000006.
// =============================================================================

use crate::DomainError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TicketStatus {
    Open,
    InProgress,
    Resolved,
    Closed,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportTicket {
    pub id: uuid::Uuid,
    pub customer_id: uuid::Uuid,
    pub order_id: Option<uuid::Uuid>,
    pub subject: String,
    pub message: String,
    pub status: TicketStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
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
}
