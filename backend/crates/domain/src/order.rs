// =============================================================================
// File:           backend/crates/domain/src/order.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Order aggregate with explicit lifecycle transitions.
//
// Specification:
//   QAS-000002, QAS-000003, QLS-000002, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;
use crate::location::Location;
use crate::money::Money;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OrderId(pub Uuid);

impl OrderId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for OrderId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address {
    pub location: Location,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    CourierAssigned,
    InTransit,
    Delivered,
    Cancelled,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::CourierAssigned => "courier_assigned",
            Self::InTransit => "in_transit",
            Self::Delivered => "delivered",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn can_assign_courier(&self) -> bool {
        matches!(self, Self::Pending)
    }

    pub fn can_start_transit(&self) -> bool {
        matches!(self, Self::CourierAssigned)
    }

    pub fn can_deliver(&self) -> bool {
        matches!(self, Self::InTransit)
    }

    pub fn can_cancel(&self) -> bool {
        matches!(self, Self::Pending | Self::CourierAssigned)
    }
}

impl std::str::FromStr for OrderStatus {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pending" => Ok(Self::Pending),
            "courier_assigned" => Ok(Self::CourierAssigned),
            "in_transit" => Ok(Self::InTransit),
            "delivered" => Ok(Self::Delivered),
            "cancelled" => Ok(Self::Cancelled),
            other => Err(DomainError::validation(format!(
                "unknown order status: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for OrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Order {
    pub id: OrderId,
    pub customer_id: Uuid,
    pub pickup: Address,
    pub dropoff: Address,
    pub status: OrderStatus,
    pub fare: Money,
    pub assigned_courier_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
}

impl Order {
    pub fn create(
        id: OrderId,
        customer_id: Uuid,
        pickup: Address,
        dropoff: Address,
        fare: Money,
        now: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if customer_id.is_nil() {
            return Err(DomainError::validation("customer id is required"));
        }
        Ok(Self {
            id,
            customer_id,
            pickup,
            dropoff,
            status: OrderStatus::Pending,
            fare,
            assigned_courier_id: None,
            created_at: now,
            delivered_at: None,
        })
    }

    pub fn assign_courier(&mut self, courier_id: Uuid) -> Result<(), DomainError> {
        if !self.status.can_assign_courier() {
            return Err(DomainError::invalid_transition(format!(
                "cannot assign a courier to an order in status {:?}",
                self.status
            )));
        }
        if courier_id.is_nil() {
            return Err(DomainError::validation("courier id is required"));
        }
        self.assigned_courier_id = Some(courier_id);
        self.status = OrderStatus::CourierAssigned;
        Ok(())
    }

    pub fn start_transit(&mut self) -> Result<(), DomainError> {
        if !self.status.can_start_transit() {
            return Err(DomainError::invalid_transition(format!(
                "cannot start transit from status {:?}",
                self.status
            )));
        }
        self.status = OrderStatus::InTransit;
        Ok(())
    }

    pub fn deliver(&mut self, now: DateTime<Utc>) -> Result<(), DomainError> {
        if !self.status.can_deliver() {
            return Err(DomainError::invalid_transition(format!(
                "cannot deliver an order in status {:?}",
                self.status
            )));
        }
        self.status = OrderStatus::Delivered;
        self.delivered_at = Some(now);
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), DomainError> {
        if !self.status.can_cancel() {
            return Err(DomainError::invalid_transition(format!(
                "cannot cancel an order in status {:?}",
                self.status
            )));
        }
        self.status = OrderStatus::Cancelled;
        Ok(())
    }

    pub fn assigned_courier(&self) -> Result<Uuid, DomainError> {
        self.assigned_courier_id
            .ok_or_else(|| DomainError::NotFound("order has no assigned courier".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::Location;

    fn sample_address(label: &str) -> Address {
        Address {
            location: Location::new(41.0, 29.0).unwrap(),
            label: Some(label.to_string()),
        }
    }

    fn sample_order() -> Order {
        Order::create(
            OrderId::new(),
            Uuid::now_v7(),
            sample_address("pickup"),
            sample_address("dropoff"),
            Money::new(1_500, "TRY").unwrap(),
            Utc::now(),
        )
        .expect("valid order")
    }

    #[test]
    fn created_order_starts_pending_without_courier() {
        let order = sample_order();
        assert_eq!(order.status, OrderStatus::Pending);
        assert!(order.assigned_courier_id.is_none());
    }

    #[test]
    fn assign_then_transit_then_deliver_is_valid_path() {
        let mut order = sample_order();
        let courier_id = Uuid::now_v7();

        order.assign_courier(courier_id).expect("assign");
        assert_eq!(order.status, OrderStatus::CourierAssigned);
        assert_eq!(order.assigned_courier_id, Some(courier_id));

        order.start_transit().expect("start transit");
        assert_eq!(order.status, OrderStatus::InTransit);

        order.deliver(Utc::now()).expect("deliver");
        assert_eq!(order.status, OrderStatus::Delivered);
        assert!(order.delivered_at.is_some());
    }

    #[test]
    fn cannot_deliver_before_pickup() {
        let mut order = sample_order();
        order.assign_courier(Uuid::now_v7()).expect("assign");
        let err = order
            .deliver(Utc::now())
            .expect_err("direct delivery must fail");
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn cannot_assign_courier_twice() {
        let mut order = sample_order();
        order.assign_courier(Uuid::now_v7()).expect("assign once");
        let err = order.assign_courier(Uuid::now_v7()).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn cannot_deliver_a_pending_order() {
        let mut order = sample_order();
        let err = order.deliver(Utc::now()).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn cancelled_order_cannot_be_delivered() {
        let mut order = sample_order();
        order.cancel().expect("cancel");
        let err = order.deliver(Utc::now()).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn rejected_courier_assignment_is_not_saved() {
        let mut order = sample_order();
        let err = order.assign_courier(Uuid::nil()).unwrap_err();
        assert!(matches!(err, DomainError::Validation(_)));
        assert!(order.assigned_courier_id.is_none());
        assert_eq!(order.status, OrderStatus::Pending);
    }
}
