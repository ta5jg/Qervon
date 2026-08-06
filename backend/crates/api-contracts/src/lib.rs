// =============================================================================
// File:           backend/crates/api-contracts/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Qervon API wire contracts: request and response DTOs for the vertical slice.
//
// Specification:
//   QAS-000005, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use qervon_domain::{
    Address, Assignment, AssignmentStatus, Courier, CourierStatus, Location, Money, Order,
    OrderStatus, VehicleType,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------- Requests ----------

#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderRequest {
    pub customer_id: Uuid,
    pub pickup: AddressDto,
    pub dropoff: AddressDto,
    pub fare_amount_minor: i64,
    pub fare_currency: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssignCourierRequest {
    /// When omitted, the closest available courier is selected automatically.
    pub courier_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterCourierRequest {
    pub id: Option<Uuid>,
    pub name: String,
    pub vehicle: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateLocationRequest {
    pub latitude: f64,
    pub longitude: f64,
}

// ---------- Value-object DTOs ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressDto {
    pub latitude: f64,
    pub longitude: f64,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoneyDto {
    pub amount_minor: i64,
    pub currency: String,
}

// ---------- Responses ----------

#[derive(Debug, Clone, Serialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub pickup: AddressDto,
    pub dropoff: AddressDto,
    pub status: OrderStatus,
    pub fare: MoneyDto,
    pub assigned_courier_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CourierResponse {
    pub id: Uuid,
    pub name: String,
    pub vehicle: VehicleType,
    pub status: CourierStatus,
    pub current_location: Option<Location>,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssignmentResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub courier_id: Uuid,
    pub status: AssignmentStatus,
    pub assigned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub status: u16,
    pub title: String,
    pub detail: String,
}

// ---------- Conversions ----------

impl From<&Address> for AddressDto {
    fn from(address: &Address) -> Self {
        Self {
            latitude: address.location.latitude,
            longitude: address.location.longitude,
            label: address.label.clone(),
        }
    }
}

impl From<&Money> for MoneyDto {
    fn from(money: &Money) -> Self {
        Self {
            amount_minor: money.amount_minor,
            currency: money.currency.clone(),
        }
    }
}

impl From<&Order> for OrderResponse {
    fn from(order: &Order) -> Self {
        Self {
            id: order.id.0,
            customer_id: order.customer_id,
            pickup: (&order.pickup).into(),
            dropoff: (&order.dropoff).into(),
            status: order.status,
            fare: (&order.fare).into(),
            assigned_courier_id: order.assigned_courier_id,
            created_at: order.created_at,
            delivered_at: order.delivered_at,
        }
    }
}

impl From<&Courier> for CourierResponse {
    fn from(courier: &Courier) -> Self {
        Self {
            id: courier.id,
            name: courier.name.clone(),
            vehicle: courier.vehicle,
            status: courier.status,
            current_location: courier.current_location,
            registered_at: courier.registered_at,
        }
    }
}

impl From<&Assignment> for AssignmentResponse {
    fn from(assignment: &Assignment) -> Self {
        Self {
            id: assignment.id,
            order_id: assignment.order_id.0,
            courier_id: assignment.courier_id,
            status: assignment.status,
            assigned_at: assignment.assigned_at,
        }
    }
}
