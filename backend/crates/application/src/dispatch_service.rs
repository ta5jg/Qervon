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
    Assignment, AssignmentRepository, CourierRepository, CourierStatus, Order, OrderId,
    OrderRepository,
};
use uuid::Uuid;

use crate::error::ApplicationError;

/// Outcome of polling for a courier's pending offer, distinguishing "never
/// had one" from "had one, but it just expired" so the caller can decide
/// whether a re-offer cascade should be attempted.
pub enum PendingOfferLookup {
    Active(Assignment, Box<Order>),
    None,
    JustExpired(Assignment),
}

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

    /// Offers a pending order to the best-ranked available courier, without
    /// changing the order's or courier's state. The courier must explicitly
    /// `accept_offer`/`reject_offer` (or let the offer expire) before the
    /// order actually becomes assigned. Returns `Ok(None)` if no eligible
    /// courier is currently available (the order simply stays `Pending`).
    pub async fn offer_for_order(
        &self,
        order_id: OrderId,
    ) -> Result<Option<Assignment>, ApplicationError> {
        let candidates = self.couriers.list_available().await?;
        self.offer_from_candidates(order_id, &candidates).await
    }

    /// Same as `offer_for_order`, but ranks a caller-scoped candidate set
    /// (used by the HTTP layer after applying tenant filtering — see
    /// `auto_assign_from_candidates`).
    pub async fn offer_from_candidates(
        &self,
        order_id: OrderId,
        candidates: &[qervon_domain::Courier],
    ) -> Result<Option<Assignment>, ApplicationError> {
        let order = self.require_order(order_id).await?;
        let ranked = crate::AiDispatcher::rank_candidates(candidates, &order.pickup.location);
        let Some(best) = ranked.first().map(|score| score.courier_id) else {
            return Ok(None);
        };
        let assignment = Assignment::offer(order_id, best, Utc::now())?;
        self.assignments.create(&assignment).await?;
        Ok(Some(assignment))
    }

    /// Returns the courier's pending offer together with the order it
    /// refers to, or `None` if there isn't one. An expired-but-still-marked-
    /// `Offered` row is lazily converted to `Cancelled` here and reported as
    /// `None`, so the courier never sees a stale offer.
    pub async fn find_pending_offer(
        &self,
        courier_id: Uuid,
    ) -> Result<Option<(Assignment, Order)>, ApplicationError> {
        match self.find_pending_offer_or_expiry(courier_id).await? {
            PendingOfferLookup::Active(assignment, order) => Ok(Some((assignment, *order))),
            PendingOfferLookup::None | PendingOfferLookup::JustExpired(_) => Ok(None),
        }
    }

    /// Same as `find_pending_offer`, but additionally distinguishes "never
    /// had an offer" from "had one, but it just expired" — the HTTP layer
    /// uses the latter to trigger a re-offer cascade to the next-best
    /// candidate (see `reoffer_from_candidates`).
    pub async fn find_pending_offer_or_expiry(
        &self,
        courier_id: Uuid,
    ) -> Result<PendingOfferLookup, ApplicationError> {
        let Some(mut assignment) = self
            .assignments
            .find_pending_offer_for_courier(courier_id)
            .await?
        else {
            return Ok(PendingOfferLookup::None);
        };
        let now = Utc::now();
        if assignment.is_offer_expired(now) {
            assignment.reject(now)?;
            self.assignments.update(&assignment).await?;
            return Ok(PendingOfferLookup::JustExpired(assignment));
        }
        let order = self.require_order(assignment.order_id).await?;
        Ok(PendingOfferLookup::Active(assignment, Box::new(order)))
    }

    /// Accepts a pending offer: the order transitions to `CourierAssigned`
    /// and the courier becomes `Busy`, exactly as an instant `assign` would,
    /// but only once the courier has explicitly agreed.
    pub async fn accept_offer(
        &self,
        order_id: OrderId,
        courier_id: Uuid,
    ) -> Result<Order, ApplicationError> {
        let mut assignment = self.require_assignment(order_id).await?;
        Self::require_offer_owner(&assignment, courier_id)?;

        assignment.accept(Utc::now())?;
        let mut order = self.require_order(order_id).await?;
        order.assign_courier(courier_id)?;
        let mut courier = self.require_courier(courier_id).await?;
        courier.go_busy()?;

        self.orders.update(&order).await?;
        self.couriers.update(&courier).await?;
        self.assignments.update(&assignment).await?;
        Ok(order)
    }

    /// Rejects a pending offer. The order stays `Pending` and the courier
    /// stays `Available` (neither was touched while merely offered), so
    /// nothing else needs to be unwound. Returns the now-rejected
    /// assignment so the caller (the HTTP layer, which alone knows the
    /// tenant-scoped candidate set) can attempt a re-offer cascade via
    /// `reoffer_from_candidates` — this method itself never re-offers.
    pub async fn reject_offer(
        &self,
        order_id: OrderId,
        courier_id: Uuid,
    ) -> Result<Assignment, ApplicationError> {
        let mut assignment = self.require_assignment(order_id).await?;
        Self::require_offer_owner(&assignment, courier_id)?;

        assignment.reject(Utc::now())?;
        self.assignments.update(&assignment).await?;
        Ok(assignment)
    }

    /// Offers a still-`Pending` order to the best-ranked candidate in
    /// `candidates` that is not in `excluded` — the re-offer cascade step
    /// run after a courier rejects a job offer or lets it expire (see
    /// `reject_offer`, `find_pending_offer_or_expiry`). Returns `Ok(None)`
    /// when no eligible candidate remains, in which case the order simply
    /// stays `Pending` for an operator to resolve manually — this method
    /// never loops through every remaining candidate itself, it only takes
    /// one more step; each subsequent rejection/expiry triggers another
    /// call, so the cascade unfolds lazily, one response at a time.
    pub async fn reoffer_from_candidates(
        &self,
        order_id: OrderId,
        excluded: &[Uuid],
        candidates: &[qervon_domain::Courier],
    ) -> Result<Option<Assignment>, ApplicationError> {
        let order = self.require_order(order_id).await?;
        if order.status != qervon_domain::OrderStatus::Pending {
            // Someone else (an operator, or a parallel accept) already
            // moved this order past the point where re-offering makes
            // sense; nothing to do.
            return Ok(None);
        }
        let eligible: Vec<qervon_domain::Courier> = candidates
            .iter()
            .filter(|courier| !excluded.contains(&courier.id))
            .cloned()
            .collect();
        let ranked = crate::AiDispatcher::rank_candidates(&eligible, &order.pickup.location);
        let Some(best) = ranked.first().map(|score| score.courier_id) else {
            return Ok(None);
        };
        let assignment =
            Assignment::offer_excluding(order_id, best, excluded.to_vec(), Utc::now())?;
        self.assignments.create(&assignment).await?;
        Ok(Some(assignment))
    }

    async fn require_assignment(&self, order_id: OrderId) -> Result<Assignment, ApplicationError> {
        self.assignments
            .find_by_order(order_id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    fn require_offer_owner(
        assignment: &Assignment,
        courier_id: Uuid,
    ) -> Result<(), ApplicationError> {
        if assignment.courier_id != courier_id {
            return Err(ApplicationError::Conflict(
                "this job was not offered to you".into(),
            ));
        }
        Ok(())
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

    /// Records a package as returned, either because the courier could not
    /// complete a mid-route drop-off or because the recipient later refused
    /// a delivered package. Only releases the courier back to `Available`
    /// when it is still `Busy` on this order (an already-delivered order's
    /// courier was freed at delivery time).
    pub async fn return_order(&self, order_id: OrderId) -> Result<Order, ApplicationError> {
        let mut order = self.require_order(order_id).await?;
        let released_courier = order.assigned_courier_id;

        order.return_order(Utc::now())?;
        if let Some(courier_id) = released_courier {
            if let Some(mut courier) = self.couriers.find_by_id(courier_id).await? {
                if courier.status == CourierStatus::Busy {
                    courier.go_available()?;
                    self.couriers.update(&courier).await?;
                }
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
