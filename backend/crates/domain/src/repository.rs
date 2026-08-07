// =============================================================================
// File:           backend/crates/domain/src/repository.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Repository ports owned by the domain. Adapters live in infrastructure.
//
// Specification:
//   QAS-000002, QAS-000006, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use async_trait::async_trait;
use std::sync::Arc;
use uuid::Uuid;

use crate::billing::{CourierPayout, Invoice, InvoiceId};
use crate::courier::Courier;
use crate::customer::{CustomerId, CustomerProfile};
use crate::dispatch::Assignment;
use crate::error::DomainError;
use crate::fleet::{Vehicle, VehicleId};
use crate::notification::{Notification, NotificationId};
use crate::order::{Order, OrderId};
use crate::tracking::{TrackingPoint, TrackingSession};
use crate::user::{User, UserId};

// ---------------------------------------------------------------------------
// OrderRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn create(&self, order: &Order) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, DomainError>;
    async fn update(&self, order: &Order) -> Result<(), DomainError>;
    async fn list_all(&self) -> Result<Vec<Order>, DomainError>;
}

#[async_trait]
impl OrderRepository for Arc<dyn OrderRepository> {
    async fn create(&self, order: &Order) -> Result<(), DomainError> {
        (**self).create(order).await
    }
    async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, DomainError> {
        (**self).find_by_id(id).await
    }
    async fn update(&self, order: &Order) -> Result<(), DomainError> {
        (**self).update(order).await
    }
    async fn list_all(&self) -> Result<Vec<Order>, DomainError> {
        (**self).list_all().await
    }
}

// ---------------------------------------------------------------------------
// CourierRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait CourierRepository: Send + Sync {
    async fn create(&self, courier: &Courier) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Courier>, DomainError>;
    async fn list_available(&self) -> Result<Vec<Courier>, DomainError>;
    async fn update(&self, courier: &Courier) -> Result<(), DomainError>;
}

#[async_trait]
impl CourierRepository for Arc<dyn CourierRepository> {
    async fn create(&self, courier: &Courier) -> Result<(), DomainError> {
        (**self).create(courier).await
    }
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Courier>, DomainError> {
        (**self).find_by_id(id).await
    }
    async fn list_available(&self) -> Result<Vec<Courier>, DomainError> {
        (**self).list_available().await
    }
    async fn update(&self, courier: &Courier) -> Result<(), DomainError> {
        (**self).update(courier).await
    }
}

// ---------------------------------------------------------------------------
// AssignmentRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait AssignmentRepository: Send + Sync {
    async fn create(&self, assignment: &Assignment) -> Result<(), DomainError>;
    async fn find_by_order(&self, order_id: OrderId) -> Result<Option<Assignment>, DomainError>;
    async fn update(&self, assignment: &Assignment) -> Result<(), DomainError>;
}

#[async_trait]
impl AssignmentRepository for Arc<dyn AssignmentRepository> {
    async fn create(&self, assignment: &Assignment) -> Result<(), DomainError> {
        (**self).create(assignment).await
    }
    async fn find_by_order(&self, order_id: OrderId) -> Result<Option<Assignment>, DomainError> {
        (**self).find_by_order(order_id).await
    }
    async fn update(&self, assignment: &Assignment) -> Result<(), DomainError> {
        (**self).update(assignment).await
    }
}

// ---------------------------------------------------------------------------
// TrackingRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait TrackingRepository: Send + Sync {
    async fn record_point(&self, point: &TrackingPoint) -> Result<(), DomainError>;
    async fn create_session(&self, session: &TrackingSession) -> Result<(), DomainError>;
    async fn find_session(&self, id: Uuid) -> Result<Option<TrackingSession>, DomainError>;
    async fn find_active_session_for_courier(
        &self,
        courier_id: Uuid,
    ) -> Result<Option<TrackingSession>, DomainError>;
    async fn update_session(&self, session: &TrackingSession) -> Result<(), DomainError>;
}

#[async_trait]
impl TrackingRepository for Arc<dyn TrackingRepository> {
    async fn record_point(&self, point: &TrackingPoint) -> Result<(), DomainError> {
        (**self).record_point(point).await
    }
    async fn create_session(&self, session: &TrackingSession) -> Result<(), DomainError> {
        (**self).create_session(session).await
    }
    async fn find_session(&self, id: Uuid) -> Result<Option<TrackingSession>, DomainError> {
        (**self).find_session(id).await
    }
    async fn find_active_session_for_courier(
        &self,
        courier_id: Uuid,
    ) -> Result<Option<TrackingSession>, DomainError> {
        (**self).find_active_session_for_courier(courier_id).await
    }
    async fn update_session(&self, session: &TrackingSession) -> Result<(), DomainError> {
        (**self).update_session(session).await
    }
}

// ---------------------------------------------------------------------------
// VehicleRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait VehicleRepository: Send + Sync {
    async fn create(&self, vehicle: &Vehicle) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: VehicleId) -> Result<Option<Vehicle>, DomainError>;
    async fn find_by_plate(&self, plate: &str) -> Result<Option<Vehicle>, DomainError>;
    async fn list_active(&self) -> Result<Vec<Vehicle>, DomainError>;
    async fn update(&self, vehicle: &Vehicle) -> Result<(), DomainError>;
}

#[async_trait]
impl VehicleRepository for Arc<dyn VehicleRepository> {
    async fn create(&self, vehicle: &Vehicle) -> Result<(), DomainError> {
        (**self).create(vehicle).await
    }
    async fn find_by_id(&self, id: VehicleId) -> Result<Option<Vehicle>, DomainError> {
        (**self).find_by_id(id).await
    }
    async fn find_by_plate(&self, plate: &str) -> Result<Option<Vehicle>, DomainError> {
        (**self).find_by_plate(plate).await
    }
    async fn list_active(&self) -> Result<Vec<Vehicle>, DomainError> {
        (**self).list_active().await
    }
    async fn update(&self, vehicle: &Vehicle) -> Result<(), DomainError> {
        (**self).update(vehicle).await
    }
}

// ---------------------------------------------------------------------------
// InvoiceRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait InvoiceRepository: Send + Sync {
    async fn create(&self, invoice: &Invoice) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: InvoiceId) -> Result<Option<Invoice>, DomainError>;
    async fn find_by_order(&self, order_id: OrderId) -> Result<Option<Invoice>, DomainError>;
    async fn update(&self, invoice: &Invoice) -> Result<(), DomainError>;
}

#[async_trait]
impl InvoiceRepository for Arc<dyn InvoiceRepository> {
    async fn create(&self, invoice: &Invoice) -> Result<(), DomainError> {
        (**self).create(invoice).await
    }
    async fn find_by_id(&self, id: InvoiceId) -> Result<Option<Invoice>, DomainError> {
        (**self).find_by_id(id).await
    }
    async fn find_by_order(&self, order_id: OrderId) -> Result<Option<Invoice>, DomainError> {
        (**self).find_by_order(order_id).await
    }
    async fn update(&self, invoice: &Invoice) -> Result<(), DomainError> {
        (**self).update(invoice).await
    }
}

// ---------------------------------------------------------------------------
// CourierPayoutRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait CourierPayoutRepository: Send + Sync {
    async fn create(&self, payout: &CourierPayout) -> Result<(), DomainError>;
    async fn find_by_courier(&self, courier_id: Uuid) -> Result<Vec<CourierPayout>, DomainError>;
    async fn update(&self, payout: &CourierPayout) -> Result<(), DomainError>;
}

#[async_trait]
impl CourierPayoutRepository for Arc<dyn CourierPayoutRepository> {
    async fn create(&self, payout: &CourierPayout) -> Result<(), DomainError> {
        (**self).create(payout).await
    }
    async fn find_by_courier(&self, courier_id: Uuid) -> Result<Vec<CourierPayout>, DomainError> {
        (**self).find_by_courier(courier_id).await
    }
    async fn update(&self, payout: &CourierPayout) -> Result<(), DomainError> {
        (**self).update(payout).await
    }
}

// ---------------------------------------------------------------------------
// NotificationRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait NotificationRepository: Send + Sync {
    async fn create(&self, notification: &Notification) -> Result<(), DomainError>;
    async fn find_by_id(
        &self,
        id: NotificationId,
    ) -> Result<Option<Notification>, DomainError>;
    async fn list_for_recipient(
        &self,
        recipient_id: Uuid,
    ) -> Result<Vec<Notification>, DomainError>;
    async fn update(&self, notification: &Notification) -> Result<(), DomainError>;
}

#[async_trait]
impl NotificationRepository for Arc<dyn NotificationRepository> {
    async fn create(&self, notification: &Notification) -> Result<(), DomainError> {
        (**self).create(notification).await
    }
    async fn find_by_id(
        &self,
        id: NotificationId,
    ) -> Result<Option<Notification>, DomainError> {
        (**self).find_by_id(id).await
    }
    async fn list_for_recipient(
        &self,
        recipient_id: Uuid,
    ) -> Result<Vec<Notification>, DomainError> {
        (**self).list_for_recipient(recipient_id).await
    }
    async fn update(&self, notification: &Notification) -> Result<(), DomainError> {
        (**self).update(notification).await
    }
}

// ---------------------------------------------------------------------------
// UserRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create(&self, user: &User) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError>;
    async fn update(&self, user: &User) -> Result<(), DomainError>;
}

#[async_trait]
impl UserRepository for Arc<dyn UserRepository> {
    async fn create(&self, user: &User) -> Result<(), DomainError> {
        (**self).create(user).await
    }
    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError> {
        (**self).find_by_id(id).await
    }
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        (**self).find_by_email(email).await
    }
    async fn update(&self, user: &User) -> Result<(), DomainError> {
        (**self).update(user).await
    }
}

// ---------------------------------------------------------------------------
// CustomerRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait CustomerRepository: Send + Sync {
    async fn create(&self, profile: &CustomerProfile) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: CustomerId) -> Result<Option<CustomerProfile>, DomainError>;
    async fn find_by_user(&self, user_id: UserId) -> Result<Option<CustomerProfile>, DomainError>;
    async fn update(&self, profile: &CustomerProfile) -> Result<(), DomainError>;
}

#[async_trait]
impl CustomerRepository for Arc<dyn CustomerRepository> {
    async fn create(&self, profile: &CustomerProfile) -> Result<(), DomainError> {
        (**self).create(profile).await
    }
    async fn find_by_id(&self, id: CustomerId) -> Result<Option<CustomerProfile>, DomainError> {
        (**self).find_by_id(id).await
    }
    async fn find_by_user(&self, user_id: UserId) -> Result<Option<CustomerProfile>, DomainError> {
        (**self).find_by_user(user_id).await
    }
    async fn update(&self, profile: &CustomerProfile) -> Result<(), DomainError> {
        (**self).update(profile).await
    }
}
