// =============================================================================
// File:           backend/modules/dispatch/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Dispatch domain module: public boundary over assignment use cases.
//
// Specification:
//   QAS-000001 through QAS-000006, QLS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use qervon_application::DispatchService;
use qervon_domain::{
    Assignment, AssignmentRepository, CourierRepository, Order, OrderId, OrderRepository,
};
use uuid::Uuid;

pub struct DispatchModule<OR, CR, AR>
where
    OR: OrderRepository,
    CR: CourierRepository,
    AR: AssignmentRepository,
{
    service: DispatchService<OR, CR, AR>,
}

impl<OR, CR, AR> DispatchModule<OR, CR, AR>
where
    OR: OrderRepository,
    CR: CourierRepository,
    AR: AssignmentRepository,
{
    pub fn new(orders: OR, couriers: CR, assignments: AR) -> Self {
        Self {
            service: DispatchService::new(orders, couriers, assignments),
        }
    }

    pub async fn assign_courier(
        &self,
        order_id: OrderId,
        courier_id: Uuid,
    ) -> Result<Assignment, qervon_application::ApplicationError> {
        self.service.assign(order_id, courier_id).await
    }

    pub async fn auto_assign(
        &self,
        order_id: OrderId,
    ) -> Result<Assignment, qervon_application::ApplicationError> {
        self.service.auto_assign(order_id).await
    }

    pub async fn start_transit(
        &self,
        order_id: OrderId,
    ) -> Result<Order, qervon_application::ApplicationError> {
        self.service.start_transit(order_id).await
    }

    pub async fn deliver_order(
        &self,
        order_id: OrderId,
    ) -> Result<Order, qervon_application::ApplicationError> {
        self.service.deliver(order_id).await
    }

    pub async fn cancel_order(
        &self,
        order_id: OrderId,
    ) -> Result<Order, qervon_application::ApplicationError> {
        self.service.cancel(order_id).await
    }
}
