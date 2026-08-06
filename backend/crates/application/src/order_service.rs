// =============================================================================
// File:           backend/crates/application/src/order_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Order intake use cases: create and read orders.
//
// Specification:
//   QAS-000002, QLS-000002, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::Utc;
use qervon_domain::{Address, Money, Order, OrderId, OrderRepository};
use uuid::Uuid;

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct CreateOrderInput {
    pub customer_id: Uuid,
    pub pickup: Address,
    pub dropoff: Address,
    pub fare: Money,
}

pub struct OrderService<R>
where
    R: OrderRepository,
{
    orders: R,
}

impl<R> OrderService<R>
where
    R: OrderRepository,
{
    pub fn new(orders: R) -> Self {
        Self { orders }
    }

    pub async fn create(&self, input: CreateOrderInput) -> Result<Order, ApplicationError> {
        let order = Order::create(
            OrderId::new(),
            input.customer_id,
            input.pickup,
            input.dropoff,
            input.fare,
            Utc::now(),
        )?;
        self.orders.create(&order).await?;
        Ok(order)
    }

    pub async fn get(&self, id: OrderId) -> Result<Order, ApplicationError> {
        self.orders
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }
}
