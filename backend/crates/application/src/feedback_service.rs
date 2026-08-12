// =============================================================================
// File:           backend/crates/application/src/feedback_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Customer feedback use cases: rating a delivered/returned order and
//   raising/managing support tickets.
//
// Specification:
//   QAS-000002, QAS-000004, QAS-000006, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::Utc;
use qervon_domain::{
    CustomerRating, CustomerRatingRepository, OrderId, OrderRepository, OrderStatus, SupportTicket,
    SupportTicketRepository, TenantId,
};
use uuid::Uuid;

use crate::error::ApplicationError;

pub struct RatingService<R, OR>
where
    R: CustomerRatingRepository,
    OR: OrderRepository,
{
    ratings: R,
    orders: OR,
}

impl<R, OR> RatingService<R, OR>
where
    R: CustomerRatingRepository,
    OR: OrderRepository,
{
    pub fn new(ratings: R, orders: OR) -> Self {
        Self { ratings, orders }
    }

    /// Rates the courier who fulfilled a delivered (or later returned)
    /// order. Only the order's own customer may rate it, only once, and
    /// only after the delivery attempt has concluded.
    pub async fn rate_order(
        &self,
        order_id: OrderId,
        customer_id: Uuid,
        rating_stars: u8,
        comment: Option<String>,
    ) -> Result<CustomerRating, ApplicationError> {
        let order = self
            .orders
            .find_by_id(order_id)
            .await?
            .ok_or(ApplicationError::NotFound)?;
        if order.customer_id != customer_id {
            return Err(ApplicationError::Conflict(
                "order does not belong to this customer".into(),
            ));
        }
        if !matches!(order.status, OrderStatus::Delivered | OrderStatus::Returned) {
            return Err(ApplicationError::Conflict(
                "only a delivered or returned order can be rated".into(),
            ));
        }
        let courier_id = order.assigned_courier_id.ok_or_else(|| {
            ApplicationError::Conflict("order has no assigned courier to rate".into())
        })?;
        if self.ratings.find_by_order(order_id.0).await?.is_some() {
            return Err(ApplicationError::Conflict(
                "this order already has a rating".into(),
            ));
        }
        let rating =
            CustomerRating::new(order_id.0, customer_id, courier_id, rating_stars, comment)?;
        self.ratings.create(&rating).await?;
        Ok(rating)
    }

    pub async fn list_for_courier(
        &self,
        courier_id: Uuid,
    ) -> Result<Vec<CustomerRating>, ApplicationError> {
        Ok(self.ratings.list_for_courier(courier_id).await?)
    }
}

pub struct SupportTicketService<R>
where
    R: SupportTicketRepository,
{
    tickets: R,
}

impl<R> SupportTicketService<R>
where
    R: SupportTicketRepository,
{
    pub fn new(tickets: R) -> Self {
        Self { tickets }
    }

    pub async fn open_ticket(
        &self,
        tenant_id: TenantId,
        customer_id: Uuid,
        order_id: Option<Uuid>,
        subject: String,
        message: String,
    ) -> Result<SupportTicket, ApplicationError> {
        let ticket = SupportTicket::open(
            tenant_id,
            customer_id,
            order_id,
            subject,
            message,
            Utc::now(),
        )?;
        self.tickets.create(&ticket).await?;
        Ok(ticket)
    }

    pub async fn list_for_customer(
        &self,
        customer_id: Uuid,
    ) -> Result<Vec<SupportTicket>, ApplicationError> {
        Ok(self.tickets.list_for_customer(customer_id).await?)
    }

    pub async fn get(&self, id: Uuid) -> Result<SupportTicket, ApplicationError> {
        self.tickets
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    pub async fn start_progress(&self, id: Uuid) -> Result<SupportTicket, ApplicationError> {
        let mut ticket = self.get(id).await?;
        ticket.start_progress()?;
        self.tickets.update(&ticket).await?;
        Ok(ticket)
    }

    pub async fn resolve(&self, id: Uuid) -> Result<SupportTicket, ApplicationError> {
        let mut ticket = self.get(id).await?;
        ticket.resolve()?;
        self.tickets.update(&ticket).await?;
        Ok(ticket)
    }

    pub async fn close(&self, id: Uuid) -> Result<SupportTicket, ApplicationError> {
        let mut ticket = self.get(id).await?;
        ticket.close()?;
        self.tickets.update(&ticket).await?;
        Ok(ticket)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc as ChronoUtc;
    use qervon_domain::{Address, Location, Money, Order};
    use qervon_infrastructure::memory::InMemoryStore;

    fn sample_order(customer_id: Uuid) -> Order {
        let address = Address {
            location: Location::new(41.0, 29.0).unwrap(),
            label: Some("pickup".into()),
        };
        Order::create(
            OrderId::new(),
            customer_id,
            address.clone(),
            address,
            Money::new(1_500, "TRY").unwrap(),
            ChronoUtc::now(),
            None,
            None,
        )
        .expect("valid order")
    }

    #[tokio::test]
    async fn rating_requires_a_delivered_order_owned_by_the_customer() {
        let store = InMemoryStore::new();
        let orders = store.order_repository();
        let service = RatingService::new(store.customer_rating_repository(), orders.clone());

        let customer_id = Uuid::now_v7();
        let mut order = sample_order(customer_id);
        orders.create(&order).await.expect("create order");

        // Not yet delivered.
        let err = service
            .rate_order(order.id, customer_id, 5, None)
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::Conflict(_)));

        let courier_id = Uuid::now_v7();
        order.assign_courier(courier_id).expect("assign");
        order.start_transit().expect("transit");
        order.deliver(ChronoUtc::now()).expect("deliver");
        orders.update(&order).await.expect("update order");

        // Wrong customer cannot rate it.
        let err = service
            .rate_order(order.id, Uuid::now_v7(), 5, None)
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::Conflict(_)));

        let rating = service
            .rate_order(order.id, customer_id, 5, Some("great".into()))
            .await
            .expect("rate order");
        assert_eq!(rating.courier_id, courier_id);

        // Cannot rate the same order twice.
        let err = service
            .rate_order(order.id, customer_id, 4, None)
            .await
            .unwrap_err();
        assert!(matches!(err, ApplicationError::Conflict(_)));

        let ratings = service
            .list_for_courier(courier_id)
            .await
            .expect("list ratings");
        assert_eq!(ratings.len(), 1);
    }

    #[tokio::test]
    async fn support_ticket_lifecycle() {
        let store = InMemoryStore::new();
        let service = SupportTicketService::new(store.support_ticket_repository());
        let tenant_id = TenantId::new();
        let customer_id = Uuid::now_v7();

        let ticket = service
            .open_ticket(
                tenant_id,
                customer_id,
                None,
                "Kurye gelmedi".into(),
                "Siparişim gecikti".into(),
            )
            .await
            .expect("open ticket");

        let in_progress = service.start_progress(ticket.id).await.expect("progress");
        assert_eq!(in_progress.status, qervon_domain::TicketStatus::InProgress);

        let resolved = service.resolve(ticket.id).await.expect("resolve");
        assert_eq!(resolved.status, qervon_domain::TicketStatus::Resolved);

        let tickets = service
            .list_for_customer(customer_id)
            .await
            .expect("list tickets");
        assert_eq!(tickets.len(), 1);
    }
}
