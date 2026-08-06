// =============================================================================
// File:           backend/modules/orders/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Orders domain module: public boundary over the order use cases.
//
// Specification:
//   QAS-000001 through QAS-000006, QLS-000002, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use qervon_application::{CreateOrderInput, OrderService};
use qervon_domain::{Order, OrderId, OrderRepository};

pub struct OrdersModule<R>
where
    R: OrderRepository,
{
    service: OrderService<R>,
}

impl<R> OrdersModule<R>
where
    R: OrderRepository,
{
    pub fn new(orders: R) -> Self {
        Self {
            service: OrderService::new(orders),
        }
    }

    pub async fn create_order(
        &self,
        input: CreateOrderInput,
    ) -> Result<Order, qervon_application::ApplicationError> {
        self.service.create(input).await
    }

    pub async fn get_order(
        &self,
        id: OrderId,
    ) -> Result<Order, qervon_application::ApplicationError> {
        self.service.get(id).await
    }
}
