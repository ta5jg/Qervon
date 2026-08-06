// =============================================================================
// File:           backend/crates/domain/src/dispatch.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Dispatch assignment aggregate linking an order to a courier.
//
// Specification:
//   QAS-000002, QAS-000003, QLS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::order::OrderId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentStatus {
    Assigned,
    Completed,
    Cancelled,
}

impl AssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Assigned => "assigned",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::str::FromStr for AssignmentStatus {
    type Err = crate::error::DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "assigned" => Ok(Self::Assigned),
            "completed" => Ok(Self::Completed),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(crate::error::DomainError::validation(format!(
                "unknown assignment status: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for AssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub id: Uuid,
    pub order_id: OrderId,
    pub courier_id: Uuid,
    pub status: AssignmentStatus,
    pub assigned_at: DateTime<Utc>,
}

impl Assignment {
    pub fn new(
        order_id: OrderId,
        courier_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Self, crate::error::DomainError> {
        if courier_id.is_nil() {
            return Err(crate::error::DomainError::validation(
                "courier id is required",
            ));
        }
        Ok(Self {
            id: Uuid::now_v7(),
            order_id,
            courier_id,
            status: AssignmentStatus::Assigned,
            assigned_at: now,
        })
    }

    pub fn complete(&mut self) {
        self.status = AssignmentStatus::Completed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignment_starts_assigned() {
        let assignment =
            Assignment::new(OrderId::new(), Uuid::now_v7(), Utc::now()).expect("valid assignment");
        assert_eq!(assignment.status, AssignmentStatus::Assigned);
    }

    #[test]
    fn rejects_nil_courier() {
        assert!(Assignment::new(OrderId::new(), Uuid::nil(), Utc::now()).is_err());
    }

    #[test]
    fn can_be_completed() {
        let mut assignment =
            Assignment::new(OrderId::new(), Uuid::now_v7(), Utc::now()).expect("valid assignment");
        assignment.complete();
        assert_eq!(assignment.status, AssignmentStatus::Completed);
    }
}
