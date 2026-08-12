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
    Assignment, AssignmentRepository, Courier, CourierRepository, Order, OrderId, OrderRepository,
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

    pub async fn auto_assign_from_candidates(
        &self,
        order_id: OrderId,
        candidates: &[Courier],
    ) -> Result<Assignment, qervon_application::ApplicationError> {
        self.service
            .auto_assign_from_candidates(order_id, candidates)
            .await
    }

    pub async fn offer_for_order(
        &self,
        order_id: OrderId,
    ) -> Result<Option<Assignment>, qervon_application::ApplicationError> {
        self.service.offer_for_order(order_id).await
    }

    pub async fn offer_from_candidates(
        &self,
        order_id: OrderId,
        candidates: &[Courier],
    ) -> Result<Option<Assignment>, qervon_application::ApplicationError> {
        self.service
            .offer_from_candidates(order_id, candidates)
            .await
    }

    pub async fn find_pending_offer(
        &self,
        courier_id: Uuid,
    ) -> Result<Option<(Assignment, Order)>, qervon_application::ApplicationError> {
        self.service.find_pending_offer(courier_id).await
    }

    pub async fn find_pending_offer_or_expiry(
        &self,
        courier_id: Uuid,
    ) -> Result<qervon_application::PendingOfferLookup, qervon_application::ApplicationError> {
        self.service.find_pending_offer_or_expiry(courier_id).await
    }

    pub async fn accept_offer(
        &self,
        order_id: OrderId,
        courier_id: Uuid,
    ) -> Result<Order, qervon_application::ApplicationError> {
        self.service.accept_offer(order_id, courier_id).await
    }

    pub async fn reject_offer(
        &self,
        order_id: OrderId,
        courier_id: Uuid,
    ) -> Result<Assignment, qervon_application::ApplicationError> {
        self.service.reject_offer(order_id, courier_id).await
    }

    /// See `DispatchService::reoffer_from_candidates`.
    pub async fn reoffer_from_candidates(
        &self,
        order_id: OrderId,
        excluded: &[Uuid],
        candidates: &[Courier],
    ) -> Result<Option<Assignment>, qervon_application::ApplicationError> {
        self.service
            .reoffer_from_candidates(order_id, excluded, candidates)
            .await
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

    pub async fn return_order(
        &self,
        order_id: OrderId,
    ) -> Result<Order, qervon_application::ApplicationError> {
        self.service.return_order(order_id).await
    }

    pub async fn cancel_order(
        &self,
        order_id: OrderId,
    ) -> Result<Order, qervon_application::ApplicationError> {
        self.service.cancel(order_id).await
    }
}
