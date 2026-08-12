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

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::order::OrderId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssignmentStatus {
    /// Offered to a courier who has not yet responded. The order stays
    /// `Pending` and the courier stays `Available` while in this state, so
    /// a rejection or expiry has no other state to unwind.
    Offered,
    Assigned,
    Completed,
    Cancelled,
}

impl AssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Offered => "offered",
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
            "offered" => Ok(Self::Offered),
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

/// How long a courier has to respond to an offered job before it expires.
pub const OFFER_TTL: Duration = Duration::seconds(45);

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assignment {
    pub id: Uuid,
    pub order_id: OrderId,
    pub courier_id: Uuid,
    pub status: AssignmentStatus,
    /// When this row was last (re)initiated: creation time for `new`, or the
    /// moment an offer became `Assigned` for `accept`.
    pub assigned_at: DateTime<Utc>,
    /// When the current offer (if any) was made. Only meaningful while
    /// `status == Offered`, but kept for audit after a response.
    pub offered_at: DateTime<Utc>,
    /// When the courier accepted or rejected the offer, if they ever did.
    pub responded_at: Option<DateTime<Utc>>,
    /// Couriers already offered this order (via `offer`/`re_offer`) who
    /// rejected or let the offer expire, carried forward across each
    /// re-offer for the same order so the re-offer cascade (see
    /// `DispatchService::reoffer_from_candidates`) never offers the same
    /// job to the same courier twice.
    pub excluded_courier_ids: Vec<Uuid>,
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
            offered_at: now,
            responded_at: None,
            excluded_courier_ids: Vec::new(),
        })
    }

    /// Creates a pending offer for a courier to accept or reject. Unlike
    /// `new`, this does not imply the order or courier have changed state —
    /// the caller (`DispatchService`) is responsible for leaving the order
    /// `Pending` and the courier `Available` until a response arrives.
    pub fn offer(
        order_id: OrderId,
        courier_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<Self, crate::error::DomainError> {
        Self::offer_excluding(order_id, courier_id, Vec::new(), now)
    }

    /// Same as `offer`, but records `excluded_courier_ids` as the set of
    /// couriers this order has already been offered to and who
    /// rejected/expired — used by the re-offer cascade
    /// (`DispatchService::reoffer_from_candidates`) so a courier is never
    /// offered the same job twice.
    pub fn offer_excluding(
        order_id: OrderId,
        courier_id: Uuid,
        excluded_courier_ids: Vec<Uuid>,
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
            status: AssignmentStatus::Offered,
            assigned_at: now,
            offered_at: now,
            responded_at: None,
            excluded_courier_ids,
        })
    }

    pub fn is_offer_expired(&self, now: DateTime<Utc>) -> bool {
        self.status == AssignmentStatus::Offered && now >= self.offered_at + OFFER_TTL
    }

    /// Accepts a pending, unexpired offer.
    pub fn accept(&mut self, now: DateTime<Utc>) -> Result<(), crate::error::DomainError> {
        if self.status != AssignmentStatus::Offered {
            return Err(crate::error::DomainError::invalid_transition(format!(
                "cannot accept a {} assignment",
                self.status
            )));
        }
        if self.is_offer_expired(now) {
            return Err(crate::error::DomainError::invalid_transition(
                "offer has expired",
            ));
        }
        self.status = AssignmentStatus::Assigned;
        self.assigned_at = now;
        self.responded_at = Some(now);
        Ok(())
    }

    /// Rejects a pending offer (expired offers can also be rejected; this is
    /// how a lazily-discovered expiry gets persisted).
    pub fn reject(&mut self, now: DateTime<Utc>) -> Result<(), crate::error::DomainError> {
        if self.status != AssignmentStatus::Offered {
            return Err(crate::error::DomainError::invalid_transition(format!(
                "cannot reject a {} assignment",
                self.status
            )));
        }
        self.status = AssignmentStatus::Cancelled;
        self.responded_at = Some(now);
        Ok(())
    }

    pub fn complete(&mut self) {
        self.status = AssignmentStatus::Completed;
    }

    /// The full set of couriers this order should never be re-offered to
    /// next: everyone already excluded on this row, plus the courier this
    /// row was offered to (who just rejected or expired). Used to seed the
    /// next `offer_excluding` call in a re-offer cascade.
    pub fn excluded_including_self(&self) -> Vec<Uuid> {
        let mut excluded = self.excluded_courier_ids.clone();
        if !excluded.contains(&self.courier_id) {
            excluded.push(self.courier_id);
        }
        excluded
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

    #[test]
    fn offer_starts_offered_and_can_be_accepted() {
        let now = Utc::now();
        let mut offer =
            Assignment::offer(OrderId::new(), Uuid::now_v7(), now).expect("valid offer");
        assert_eq!(offer.status, AssignmentStatus::Offered);
        assert!(!offer.is_offer_expired(now));

        offer.accept(now).expect("accept");
        assert_eq!(offer.status, AssignmentStatus::Assigned);
        assert!(offer.responded_at.is_some());
    }

    #[test]
    fn offer_can_be_rejected() {
        let now = Utc::now();
        let mut offer =
            Assignment::offer(OrderId::new(), Uuid::now_v7(), now).expect("valid offer");
        offer.reject(now).expect("reject");
        assert_eq!(offer.status, AssignmentStatus::Cancelled);
        assert!(offer.responded_at.is_some());
    }

    #[test]
    fn expired_offer_cannot_be_accepted() {
        let now = Utc::now();
        let offer = Assignment::offer(OrderId::new(), Uuid::now_v7(), now).expect("valid offer");
        let later = now + OFFER_TTL + Duration::seconds(1);
        assert!(offer.is_offer_expired(later));

        let mut offer = offer;
        let err = offer.accept(later).unwrap_err();
        assert!(matches!(
            err,
            crate::error::DomainError::InvalidTransition(_)
        ));
    }

    #[test]
    fn expired_offer_can_still_be_rejected_to_persist_the_expiry() {
        let now = Utc::now();
        let mut offer =
            Assignment::offer(OrderId::new(), Uuid::now_v7(), now).expect("valid offer");
        let later = now + OFFER_TTL + Duration::seconds(1);
        offer.reject(later).expect("reject expired offer");
        assert_eq!(offer.status, AssignmentStatus::Cancelled);
    }

    #[test]
    fn offer_excluding_carries_the_excluded_set() {
        let now = Utc::now();
        let first_courier = Uuid::now_v7();
        let second_courier = Uuid::now_v7();
        let offer = Assignment::offer(OrderId::new(), first_courier, now).expect("valid offer");
        assert!(offer.excluded_courier_ids.is_empty());

        let mut rejected = offer;
        rejected.reject(now).expect("reject");
        let excluded = rejected.excluded_including_self();
        assert_eq!(excluded, vec![first_courier]);

        let re_offer =
            Assignment::offer_excluding(rejected.order_id, second_courier, excluded.clone(), now)
                .expect("valid re-offer");
        assert_eq!(re_offer.excluded_courier_ids, excluded);
        assert_eq!(re_offer.status, AssignmentStatus::Offered);
    }

    #[test]
    fn cannot_accept_or_reject_an_already_assigned_assignment() {
        let now = Utc::now();
        let mut assignment =
            Assignment::new(OrderId::new(), Uuid::now_v7(), now).expect("valid assignment");
        assert!(assignment.accept(now).is_err());
        assert!(assignment.reject(now).is_err());
    }
}
