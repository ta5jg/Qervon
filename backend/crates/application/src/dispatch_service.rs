// =============================================================================
// File:           backend/crates/application/src/dispatch_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Dispatch use cases: assign, auto-assign, start transit, deliver, cancel.
//
// Specification:
//   QAS-000002, QAS-000003, QLS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::Utc;
use qervon_domain::{
    Assignment, AssignmentRepository, CourierRepository, Order, OrderId, OrderRepository,
};
use uuid::Uuid;

use crate::error::ApplicationError;

pub struct DispatchService<OR, CR, AR>
where
    OR: OrderRepository,
    CR: CourierRepository,
    AR: AssignmentRepository,
{
    orders: OR,
    couriers: CR,
    assignments: AR,
}

impl<OR, CR, AR> DispatchService<OR, CR, AR>
where
    OR: OrderRepository,
    CR: CourierRepository,
    AR: AssignmentRepository,
{
    pub fn new(orders: OR, couriers: CR, assignments: AR) -> Self {
        Self {
            orders,
            couriers,
            assignments,
        }
    }

    pub async fn assign(
        &self,
        order_id: OrderId,
        courier_id: Uuid,
    ) -> Result<Assignment, ApplicationError> {
        let mut order = self.require_order(order_id).await?;
        let mut courier = self.require_courier(courier_id).await?;

        order.assign_courier(courier_id)?;
        courier.go_busy()?;
        let assignment = Assignment::new(order_id, courier_id, Utc::now())?;

        self.orders.update(&order).await?;
        self.couriers.update(&courier).await?;
        self.assignments.create(&assignment).await?;
        Ok(assignment)
    }

    pub async fn auto_assign(&self, order_id: OrderId) -> Result<Assignment, ApplicationError> {
        let candidates = self.couriers.list_available().await?;
        self.auto_assign_from_candidates(order_id, &candidates)
            .await
    }

    /// Assign the best available courier from a caller-scoped candidate set.
    ///
    /// The delivery domain itself has no tenant field, so the HTTP layer uses
    /// this entry point after applying tenant ownership filtering. Keeping the
    /// ranking here preserves one dispatch algorithm for both global system
    /// jobs and tenant-scoped interactive dispatch.
    pub async fn auto_assign_from_candidates(
        &self,
        order_id: OrderId,
        candidates: &[qervon_domain::Courier],
    ) -> Result<Assignment, ApplicationError> {
        let order = self.require_order(order_id).await?;
        let ranked = crate::AiDispatcher::rank_candidates(candidates, &order.pickup.location);
        let best =
            ranked
                .first()
                .map(|score| score.courier_id)
                .ok_or(ApplicationError::Conflict(
                    "no available courier with a known location".to_string(),
                ))?;

        self.assign(order_id, best).await
    }

    pub async fn start_transit(&self, order_id: OrderId) -> Result<Order, ApplicationError> {
        let mut order = self.require_order(order_id).await?;
        order.start_transit()?;
        self.orders.update(&order).await?;
        Ok(order)
    }

    pub async fn deliver(&self, order_id: OrderId) -> Result<Order, ApplicationError> {
        let mut order = self.require_order(order_id).await?;
        let courier_id = order.assigned_courier()?;

        order.deliver(Utc::now())?;
        let mut courier = self.require_courier(courier_id).await?;
        courier.go_available()?;

        if let Some(mut assignment) = self.assignments.find_by_order(order_id).await? {
            assignment.complete();
            self.assignments.update(&assignment).await?;
        }

        self.orders.update(&order).await?;
        self.couriers.update(&courier).await?;
        Ok(order)
    }

    pub async fn cancel(&self, order_id: OrderId) -> Result<Order, ApplicationError> {
        let mut order = self.require_order(order_id).await?;
        let released_courier = order.assigned_courier_id;

        order.cancel()?;
        if let Some(courier_id) = released_courier {
            if let Some(mut courier) = self.couriers.find_by_id(courier_id).await? {
                courier.go_available()?;
                self.couriers.update(&courier).await?;
            }
        }

        self.orders.update(&order).await?;
        Ok(order)
    }

    async fn require_order(&self, order_id: OrderId) -> Result<Order, ApplicationError> {
        self.orders
            .find_by_id(order_id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    async fn require_courier(
        &self,
        courier_id: Uuid,
    ) -> Result<qervon_domain::Courier, ApplicationError> {
        self.couriers
            .find_by_id(courier_id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }
}
