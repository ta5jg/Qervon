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
use chrono::{DateTime, Utc};
use std::sync::Arc;
use uuid::Uuid;

use crate::billing::{CourierPayout, Invoice, InvoiceId};
use crate::coupon::Coupon;
use crate::courier::Courier;
use crate::courier_wallet::{CourierWallet, WalletTransaction};
use crate::credential::{Credential, RefreshSession};
use crate::customer::{CustomerId, CustomerProfile};
use crate::customer_feedback::{CustomerRating, SupportTicket};
use crate::delivery_pricing::DeliveryPricing;
use crate::device_push_token::DevicePushToken;
use crate::dispatch::Assignment;
use crate::error::DomainError;
use crate::fleet::{Vehicle, VehicleId};
use crate::notification::{Notification, NotificationId};
use crate::order::{Order, OrderId};
use crate::otp_challenge::OtpChallenge;
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
    async fn has_any_tenant(&self) -> Result<bool, DomainError>;
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
    async fn bind_vehicle(
        &self,
        tenant_id: TenantId,
        vehicle_id: VehicleId,
    ) -> Result<(), DomainError>;
    async fn find_vehicle_tenant(
        &self,
        vehicle_id: VehicleId,
    ) -> Result<Option<TenantId>, DomainError>;
}

#[async_trait]
impl TenantRepository for Arc<dyn TenantRepository> {
    async fn create_tenant(&self, tenant: &TenantCompany, slug: &str) -> Result<(), DomainError> {
        (**self).create_tenant(tenant, slug).await
    }
    async fn has_any_tenant(&self) -> Result<bool, DomainError> {
        (**self).has_any_tenant().await
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
    async fn bind_vehicle(
        &self,
        tenant_id: TenantId,
        vehicle_id: VehicleId,
    ) -> Result<(), DomainError> {
        (**self).bind_vehicle(tenant_id, vehicle_id).await
    }
    async fn find_vehicle_tenant(
        &self,
        vehicle_id: VehicleId,
    ) -> Result<Option<TenantId>, DomainError> {
        (**self).find_vehicle_tenant(vehicle_id).await
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
    /// Creates or replaces the assignment for `assignment.order_id` (upsert
    /// by order, matching the `UNIQUE(order_id)` constraint). This lets a
    /// rejected/expired offer be re-offered to another courier, or an
    /// operator's instant manual assignment override a pending offer,
    /// without violating the one-row-per-order invariant.
    async fn create(&self, assignment: &Assignment) -> Result<(), DomainError>;
    async fn find_by_order(&self, order_id: OrderId) -> Result<Option<Assignment>, DomainError>;
    /// Returns the courier's currently pending offer, if any
    /// (`status == Offered`), regardless of expiry — callers decide how to
    /// treat an expired-but-still-`Offered` row.
    async fn find_pending_offer_for_courier(
        &self,
        courier_id: Uuid,
    ) -> Result<Option<Assignment>, DomainError>;
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
    async fn find_pending_offer_for_courier(
        &self,
        courier_id: Uuid,
    ) -> Result<Option<Assignment>, DomainError> {
        (**self).find_pending_offer_for_courier(courier_id).await
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
// CourierWalletRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait CourierWalletRepository: Send + Sync {
    async fn find_by_courier(&self, courier_id: Uuid)
        -> Result<Option<CourierWallet>, DomainError>;
    async fn create(&self, wallet: &CourierWallet) -> Result<(), DomainError>;
    /// Persists a wallet mutation as one atomic step: updates the header
    /// totals from `wallet` and appends `transaction` as a new ledger row.
    /// Callers must ensure `transaction` is the single new entry produced by
    /// the mutation that led to this `wallet` state (see
    /// `CourierWallet::add_earning`/`add_bonus`/`apply_penalty`).
    async fn append_transaction(
        &self,
        wallet: &CourierWallet,
        transaction: &WalletTransaction,
    ) -> Result<(), DomainError>;
}

#[async_trait]
impl CourierWalletRepository for Arc<dyn CourierWalletRepository> {
    async fn find_by_courier(
        &self,
        courier_id: Uuid,
    ) -> Result<Option<CourierWallet>, DomainError> {
        (**self).find_by_courier(courier_id).await
    }
    async fn create(&self, wallet: &CourierWallet) -> Result<(), DomainError> {
        (**self).create(wallet).await
    }
    async fn append_transaction(
        &self,
        wallet: &CourierWallet,
        transaction: &WalletTransaction,
    ) -> Result<(), DomainError> {
        (**self).append_transaction(wallet, transaction).await
    }
}

// ---------------------------------------------------------------------------
// CustomerRatingRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait CustomerRatingRepository: Send + Sync {
    async fn create(&self, rating: &CustomerRating) -> Result<(), DomainError>;
    async fn find_by_order(&self, order_id: Uuid) -> Result<Option<CustomerRating>, DomainError>;
    async fn list_for_courier(&self, courier_id: Uuid) -> Result<Vec<CustomerRating>, DomainError>;
}

#[async_trait]
impl CustomerRatingRepository for Arc<dyn CustomerRatingRepository> {
    async fn create(&self, rating: &CustomerRating) -> Result<(), DomainError> {
        (**self).create(rating).await
    }
    async fn find_by_order(&self, order_id: Uuid) -> Result<Option<CustomerRating>, DomainError> {
        (**self).find_by_order(order_id).await
    }
    async fn list_for_courier(&self, courier_id: Uuid) -> Result<Vec<CustomerRating>, DomainError> {
        (**self).list_for_courier(courier_id).await
    }
}

// ---------------------------------------------------------------------------
// SupportTicketRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait SupportTicketRepository: Send + Sync {
    async fn create(&self, ticket: &SupportTicket) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SupportTicket>, DomainError>;
    async fn list_for_customer(&self, customer_id: Uuid)
        -> Result<Vec<SupportTicket>, DomainError>;
    async fn list_for_tenant(&self, tenant_id: TenantId) -> Result<Vec<SupportTicket>, DomainError>;
    async fn update(&self, ticket: &SupportTicket) -> Result<(), DomainError>;
}

#[async_trait]
impl SupportTicketRepository for Arc<dyn SupportTicketRepository> {
    async fn create(&self, ticket: &SupportTicket) -> Result<(), DomainError> {
        (**self).create(ticket).await
    }
    async fn find_by_id(&self, id: Uuid) -> Result<Option<SupportTicket>, DomainError> {
        (**self).find_by_id(id).await
    }
    async fn list_for_customer(
        &self,
        customer_id: Uuid,
    ) -> Result<Vec<SupportTicket>, DomainError> {
        (**self).list_for_customer(customer_id).await
    }
    async fn list_for_tenant(&self, tenant_id: TenantId) -> Result<Vec<SupportTicket>, DomainError> {
        (**self).list_for_tenant(tenant_id).await
    }
    async fn update(&self, ticket: &SupportTicket) -> Result<(), DomainError> {
        (**self).update(ticket).await
    }
}

// ---------------------------------------------------------------------------
// CouponRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait CouponRepository: Send + Sync {
    async fn create(&self, coupon: &Coupon) -> Result<(), DomainError>;
    async fn find_by_code(
        &self,
        tenant_id: TenantId,
        code: &str,
    ) -> Result<Option<Coupon>, DomainError>;
    async fn list_for_tenant(&self, tenant_id: TenantId) -> Result<Vec<Coupon>, DomainError>;
    async fn update(&self, coupon: &Coupon) -> Result<(), DomainError>;
}

#[async_trait]
impl CouponRepository for Arc<dyn CouponRepository> {
    async fn create(&self, coupon: &Coupon) -> Result<(), DomainError> {
        (**self).create(coupon).await
    }
    async fn find_by_code(
        &self,
        tenant_id: TenantId,
        code: &str,
    ) -> Result<Option<Coupon>, DomainError> {
        (**self).find_by_code(tenant_id, code).await
    }
    async fn list_for_tenant(&self, tenant_id: TenantId) -> Result<Vec<Coupon>, DomainError> {
        (**self).list_for_tenant(tenant_id).await
    }
    async fn update(&self, coupon: &Coupon) -> Result<(), DomainError> {
        (**self).update(coupon).await
    }
}

// ---------------------------------------------------------------------------
// DevicePushTokenRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait DevicePushTokenRepository: Send + Sync {
    async fn create(&self, token: &DevicePushToken) -> Result<(), DomainError>;
    async fn find_by_user_and_token(
        &self,
        user_id: UserId,
        device_token: &str,
    ) -> Result<Option<DevicePushToken>, DomainError>;
    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<DevicePushToken>, DomainError>;
    /// Deletes a token owned by `user_id`. No-op (`Ok(())`) if it does not
    /// exist or belongs to someone else, so callers cannot probe for other
    /// users' registration ids.
    async fn delete(&self, user_id: UserId, id: Uuid) -> Result<(), DomainError>;
}

#[async_trait]
impl DevicePushTokenRepository for Arc<dyn DevicePushTokenRepository> {
    async fn create(&self, token: &DevicePushToken) -> Result<(), DomainError> {
        (**self).create(token).await
    }
    async fn find_by_user_and_token(
        &self,
        user_id: UserId,
        device_token: &str,
    ) -> Result<Option<DevicePushToken>, DomainError> {
        (**self).find_by_user_and_token(user_id, device_token).await
    }
    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<DevicePushToken>, DomainError> {
        (**self).list_for_user(user_id).await
    }
    async fn delete(&self, user_id: UserId, id: Uuid) -> Result<(), DomainError> {
        (**self).delete(user_id, id).await
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
    async fn find_by_phone(&self, phone: &str) -> Result<Option<User>, DomainError>;
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
    async fn find_by_phone(&self, phone: &str) -> Result<Option<User>, DomainError> {
        (**self).find_by_phone(phone).await
    }
    async fn update(&self, user: &User) -> Result<(), DomainError> {
        (**self).update(user).await
    }
}

// ---------------------------------------------------------------------------
// OtpChallengeRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait OtpChallengeRepository: Send + Sync {
    async fn create(&self, challenge: &OtpChallenge) -> Result<(), DomainError>;
    /// Returns the most recently created, not-yet-expired, not-yet-consumed
    /// challenge for this tenant+phone pair, if any.
    async fn find_latest_active(
        &self,
        tenant_id: TenantId,
        phone: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<OtpChallenge>, DomainError>;
    async fn update(&self, challenge: &OtpChallenge) -> Result<(), DomainError>;
}

#[async_trait]
impl OtpChallengeRepository for Arc<dyn OtpChallengeRepository> {
    async fn create(&self, challenge: &OtpChallenge) -> Result<(), DomainError> {
        (**self).create(challenge).await
    }
    async fn find_latest_active(
        &self,
        tenant_id: TenantId,
        phone: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<OtpChallenge>, DomainError> {
        (**self).find_latest_active(tenant_id, phone, now).await
    }
    async fn update(&self, challenge: &OtpChallenge) -> Result<(), DomainError> {
        (**self).update(challenge).await
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

// ---------------------------------------------------------------------------
// DeliveryPricingRepository
// ---------------------------------------------------------------------------

#[async_trait]
pub trait DeliveryPricingRepository: Send + Sync {
    async fn find_by_tenant(
        &self,
        tenant_id: TenantId,
    ) -> Result<Option<DeliveryPricing>, DomainError>;
    /// Creates or replaces the tenant's pricing configuration (one row per
    /// tenant).
    async fn upsert(&self, pricing: &DeliveryPricing) -> Result<(), DomainError>;
}

#[async_trait]
impl DeliveryPricingRepository for Arc<dyn DeliveryPricingRepository> {
    async fn find_by_tenant(
        &self,
        tenant_id: TenantId,
    ) -> Result<Option<DeliveryPricing>, DomainError> {
        (**self).find_by_tenant(tenant_id).await
    }
    async fn upsert(&self, pricing: &DeliveryPricing) -> Result<(), DomainError> {
        (**self).upsert(pricing).await
    }
}
