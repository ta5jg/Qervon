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
use crate::credential::{Credential, RefreshSession};
use crate::customer::{CustomerId, CustomerProfile};
use crate::dispatch::Assignment;
use crate::error::DomainError;
use crate::fleet::{Vehicle, VehicleId};
use crate::notification::{Notification, NotificationId};
use crate::order::{Order, OrderId};
use crate::proof_of_delivery::ProofOfDeliveryRecord;
use crate::tenant::{TenantCompany, TenantId, TenantMembership};
use crate::tracking::{TrackingPoint, TrackingSession};
use crate::user::{User, UserId};
use crate::webhook::WebhookSubscription;

#[async_trait]
pub trait CredentialRepository: Send + Sync {
    async fn save_credential(&self, credential: &Credential) -> Result<(), DomainError>;
    async fn find_credential(&self, user_id: UserId) -> Result<Option<Credential>, DomainError>;
    async fn save_refresh_session(&self, session: &RefreshSession) -> Result<(), DomainError>;
    async fn find_refresh_session(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshSession>, DomainError>;
    async fn revoke_refresh_session(&self, id: Uuid) -> Result<(), DomainError>;
}

#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn create_tenant(&self, tenant: &TenantCompany, slug: &str) -> Result<(), DomainError>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<TenantCompany>, DomainError>;
    async fn add_member(&self, membership: &TenantMembership) -> Result<(), DomainError>;
    async fn find_membership(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Option<TenantMembership>, DomainError>;
    async fn bind_courier(&self, tenant_id: TenantId, courier_id: Uuid) -> Result<(), DomainError>;
    async fn bind_order(&self, tenant_id: TenantId, order_id: OrderId) -> Result<(), DomainError>;
    async fn find_courier_tenant(&self, courier_id: Uuid) -> Result<Option<TenantId>, DomainError>;
    async fn find_order_tenant(&self, order_id: OrderId) -> Result<Option<TenantId>, DomainError>;
}

#[async_trait]
impl TenantRepository for Arc<dyn TenantRepository> {
    async fn create_tenant(&self, tenant: &TenantCompany, slug: &str) -> Result<(), DomainError> {
        (**self).create_tenant(tenant, slug).await
    }
    async fn find_by_slug(&self, slug: &str) -> Result<Option<TenantCompany>, DomainError> {
        (**self).find_by_slug(slug).await
    }
    async fn add_member(&self, membership: &TenantMembership) -> Result<(), DomainError> {
        (**self).add_member(membership).await
    }
    async fn find_membership(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Option<TenantMembership>, DomainError> {
        (**self).find_membership(tenant_id, user_id).await
    }
    async fn bind_courier(&self, tenant_id: TenantId, courier_id: Uuid) -> Result<(), DomainError> {
        (**self).bind_courier(tenant_id, courier_id).await
    }
    async fn bind_order(&self, tenant_id: TenantId, order_id: OrderId) -> Result<(), DomainError> {
        (**self).bind_order(tenant_id, order_id).await
    }
    async fn find_courier_tenant(&self, courier_id: Uuid) -> Result<Option<TenantId>, DomainError> {
        (**self).find_courier_tenant(courier_id).await
    }
    async fn find_order_tenant(&self, order_id: OrderId) -> Result<Option<TenantId>, DomainError> {
        (**self).find_order_tenant(order_id).await
    }
}

#[async_trait]
impl CredentialRepository for Arc<dyn CredentialRepository> {
    async fn save_credential(&self, credential: &Credential) -> Result<(), DomainError> {
        (**self).save_credential(credential).await
    }
    async fn find_credential(&self, user_id: UserId) -> Result<Option<Credential>, DomainError> {
        (**self).find_credential(user_id).await
    }
    async fn save_refresh_session(&self, session: &RefreshSession) -> Result<(), DomainError> {
        (**self).save_refresh_session(session).await
    }
    async fn find_refresh_session(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshSession>, DomainError> {
        (**self).find_refresh_session(token_hash).await
    }
    async fn revoke_refresh_session(&self, id: Uuid) -> Result<(), DomainError> {
        (**self).revoke_refresh_session(id).await
    }
}

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
    async fn list_all(&self) -> Result<Vec<Courier>, DomainError>;
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
    async fn list_all(&self) -> Result<Vec<Courier>, DomainError> {
        (**self).list_all().await
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
// ProofOfDeliveryRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait ProofOfDeliveryRepository: Send + Sync {
    async fn create(&self, proof: &ProofOfDeliveryRecord) -> Result<(), DomainError>;
    async fn find_by_order(
        &self,
        order_id: OrderId,
    ) -> Result<Option<ProofOfDeliveryRecord>, DomainError>;
}

#[async_trait]
impl ProofOfDeliveryRepository for Arc<dyn ProofOfDeliveryRepository> {
    async fn create(&self, proof: &ProofOfDeliveryRecord) -> Result<(), DomainError> {
        (**self).create(proof).await
    }

    async fn find_by_order(
        &self,
        order_id: OrderId,
    ) -> Result<Option<ProofOfDeliveryRecord>, DomainError> {
        (**self).find_by_order(order_id).await
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
    async fn find_by_id(&self, id: NotificationId) -> Result<Option<Notification>, DomainError>;
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
    async fn find_by_id(&self, id: NotificationId) -> Result<Option<Notification>, DomainError> {
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

#[async_trait]
pub trait WebhookRepository: Send + Sync {
    async fn create(&self, subscription: &WebhookSubscription) -> Result<(), DomainError>;
    async fn list_for_tenant(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<WebhookSubscription>, DomainError>;
    async fn delete(&self, tenant_id: TenantId, id: Uuid) -> Result<(), DomainError>;
}

#[async_trait]
impl WebhookRepository for Arc<dyn WebhookRepository> {
    async fn create(&self, subscription: &WebhookSubscription) -> Result<(), DomainError> {
        (**self).create(subscription).await
    }
    async fn list_for_tenant(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<WebhookSubscription>, DomainError> {
        (**self).list_for_tenant(tenant_id).await
    }
    async fn delete(&self, tenant_id: TenantId, id: Uuid) -> Result<(), DomainError> {
        (**self).delete(tenant_id, id).await
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
