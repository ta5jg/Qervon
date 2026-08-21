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
    Returned,
}

impl OrderStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::CourierAssigned => "courier_assigned",
            Self::InTransit => "in_transit",
            Self::Delivered => "delivered",
            Self::Cancelled => "cancelled",
            Self::Returned => "returned",
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

    /// A package can be reported returned either mid-route (the courier
    /// could not complete the drop-off and is bringing it back) or after
    /// delivery (the recipient later refuses/returns the package).
    pub fn can_return(&self) -> bool {
        matches!(self, Self::InTransit | Self::Delivered)
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
            "returned" => Ok(Self::Returned),
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

/// How the customer intends to pay. There is no real payment gateway
/// integration behind `Card`/`Qr`/`Wallet` (see BACKEND_BACKLOG.md): this
/// only records the chosen method and, for cash, whether the courier has
/// confirmed collecting it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Cash,
    Card,
    Qr,
    Wallet,
}

impl PaymentMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Cash => "cash",
            Self::Card => "card",
            Self::Qr => "qr",
            Self::Wallet => "wallet",
        }
    }
}

impl std::str::FromStr for PaymentMethod {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "cash" => Ok(Self::Cash),
            "card" => Ok(Self::Card),
            "qr" => Ok(Self::Qr),
            "wallet" => Ok(Self::Wallet),
            other => Err(DomainError::validation(format!(
                "unknown payment method: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for PaymentMethod {
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
    pub returned_at: Option<DateTime<Utc>>,
    pub payment_method: Option<PaymentMethod>,
    pub payment_collected: bool,
    /// Free-form delivery instructions from the customer (e.g. "kapıcıya
    /// bırakın"). Set once at creation; never required.
    pub delivery_note: Option<String>,
    /// A contact number for the courier to reach at the dropoff, distinct
    /// from the account holder's own phone. Set once at creation.
    pub contact_phone: Option<String>,
    /// Immutable URL of the courier's pickup photo, required before transit.
    pub pickup_photo_evidence_url: Option<String>,
}

impl Order {
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        id: OrderId,
        customer_id: Uuid,
        pickup: Address,
        dropoff: Address,
        fare: Money,
        now: DateTime<Utc>,
        delivery_note: Option<String>,
        contact_phone: Option<String>,
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
            returned_at: None,
            payment_method: None,
            payment_collected: false,
            delivery_note: delivery_note.filter(|note| !note.trim().is_empty()),
            contact_phone: contact_phone.filter(|phone| !phone.trim().is_empty()),
            pickup_photo_evidence_url: None,
        })
    }

    pub fn set_payment_method(&mut self, method: PaymentMethod) {
        self.payment_method = Some(method);
    }

    /// Confirms the fare has been collected (e.g. cash handed to the
    /// courier). Requires a payment method to already be chosen, and cannot
    /// be confirmed twice.
    pub fn mark_payment_collected(&mut self) -> Result<(), DomainError> {
        if self.payment_method.is_none() {
            return Err(DomainError::invalid_transition(
                "cannot confirm payment collection before a payment method is chosen",
            ));
        }
        if self.payment_collected {
            return Err(DomainError::invalid_transition(
                "payment has already been marked as collected",
            ));
        }
        self.payment_collected = true;
        Ok(())
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

    pub fn record_pickup_evidence(
        &mut self,
        photo_url: impl Into<String>,
    ) -> Result<(), DomainError> {
        let photo_url = photo_url.into();
        if !photo_url.starts_with("/v1/uploads/pickup-photos/")
            && !photo_url.starts_with("/v1/uploads/delivery-photos/")
        {
            return Err(DomainError::validation(
                "pickup evidence must be an uploaded pickup photo",
            ));
        }
        self.pickup_photo_evidence_url = Some(photo_url);
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

    pub fn return_order(&mut self, now: DateTime<Utc>) -> Result<(), DomainError> {
        if !self.status.can_return() {
            return Err(DomainError::invalid_transition(format!(
                "cannot return an order in status {:?}",
                self.status
            )));
        }
        self.status = OrderStatus::Returned;
        self.returned_at = Some(now);
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
            None,
            None,
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

    #[test]
    fn in_transit_order_can_be_returned() {
        let mut order = sample_order();
        order.assign_courier(Uuid::now_v7()).expect("assign");
        order.start_transit().expect("start transit");
        order.return_order(Utc::now()).expect("return");
        assert_eq!(order.status, OrderStatus::Returned);
        assert!(order.returned_at.is_some());
    }

    #[test]
    fn delivered_order_can_later_be_returned() {
        let mut order = sample_order();
        order.assign_courier(Uuid::now_v7()).expect("assign");
        order.start_transit().expect("start transit");
        order.deliver(Utc::now()).expect("deliver");
        order
            .return_order(Utc::now())
            .expect("return after delivery");
        assert_eq!(order.status, OrderStatus::Returned);
    }

    #[test]
    fn pending_order_cannot_be_returned() {
        let mut order = sample_order();
        let err = order.return_order(Utc::now()).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn payment_method_round_trip_strings() {
        for variant in [
            PaymentMethod::Cash,
            PaymentMethod::Card,
            PaymentMethod::Qr,
            PaymentMethod::Wallet,
        ] {
            assert_eq!(variant.as_str().parse::<PaymentMethod>(), Ok(variant));
        }
    }

    #[test]
    fn cannot_confirm_payment_before_a_method_is_chosen() {
        let mut order = sample_order();
        let err = order.mark_payment_collected().unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn confirming_payment_twice_is_rejected() {
        let mut order = sample_order();
        order.set_payment_method(PaymentMethod::Cash);
        order.mark_payment_collected().expect("first confirmation");
        let err = order.mark_payment_collected().unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn delivery_note_and_contact_phone_are_stored_when_present() {
        let order = Order::create(
            OrderId::new(),
            Uuid::now_v7(),
            sample_address("pickup"),
            sample_address("dropoff"),
            Money::new(1_500, "TRY").unwrap(),
            Utc::now(),
            Some("Kapıcıya bırakın".to_string()),
            Some("+905551234567".to_string()),
        )
        .expect("valid order");
        assert_eq!(order.delivery_note, Some("Kapıcıya bırakın".to_string()));
        assert_eq!(order.contact_phone, Some("+905551234567".to_string()));
    }

    #[test]
    fn blank_delivery_note_and_contact_phone_are_normalized_to_none() {
        let order = Order::create(
            OrderId::new(),
            Uuid::now_v7(),
            sample_address("pickup"),
            sample_address("dropoff"),
            Money::new(1_500, "TRY").unwrap(),
            Utc::now(),
            Some("   ".to_string()),
            Some("".to_string()),
        )
        .expect("valid order");
        assert!(order.delivery_note.is_none());
        assert!(order.contact_phone.is_none());
    }

    #[test]
    fn returned_order_cannot_be_returned_again() {
        let mut order = sample_order();
        order.assign_courier(Uuid::now_v7()).expect("assign");
        order.start_transit().expect("start transit");
        order.return_order(Utc::now()).expect("first return");
        let err = order.return_order(Utc::now()).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }
}
