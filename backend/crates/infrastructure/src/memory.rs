// =============================================================================
// File:           backend/crates/infrastructure/src/memory.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   In-memory repository adapters used by tests and local development.
//
// Specification:
//   QAS-000002, QAS-000006, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use qervon_domain::{
    Assignment, AssignmentRepository, AssignmentStatus, ColdChainTelemetry,
    ColdChainTelemetryRepository, Coupon, CouponRepository, Courier, CourierPayout,
    CourierPayoutRepository, CourierRepository, CourierStatus, CourierWallet,
    CourierWalletRepository, Credential, CredentialRepository, CustomerId, CustomerProfile,
    CustomerRating, CustomerRatingRepository, CustomerRepository, DeliveryPricing,
    DeliveryPricingRepository, DevicePushToken, DevicePushTokenRepository, DomainError,
    FieldServiceAppointment, FieldServiceAppointmentRepository, HubManifestAssignment, Invoice,
    InvoiceId, InvoiceRepository, Notification, NotificationId, NotificationRepository, Order,
    OrderId, OrderRepository, OtpChallenge, OtpChallengeRepository, ProofOfDeliveryRecord,
    ProofOfDeliveryRepository, RefreshSession, RouteBreadcrumb, RouteBreadcrumbRepository,
    SupportTicket, SupportTicketRepository, TenantCompany, TenantId, TenantMembership,
    TenantRepository, TrackingPoint, TrackingRepository, TrackingSession, TrackingSessionStatus,
    User, UserId, UserRepository, Vehicle, VehicleId, VehicleRepository, VehicleStatus,
    WalletTransaction, WarehouseHub, WarehouseHubRepository, WebhookRepository,
    WebhookSubscription,
};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// InMemoryStore — central factory for all in-memory repositories
// ---------------------------------------------------------------------------

#[derive(Clone, Default)]
pub struct InMemoryStore {
    orders: Arc<RwLock<HashMap<OrderId, Order>>>,
    couriers: Arc<RwLock<HashMap<Uuid, Courier>>>,
    assignments: Arc<RwLock<HashMap<OrderId, Assignment>>>,
    tracking_points: Arc<RwLock<Vec<TrackingPoint>>>,
    tracking_sessions: Arc<RwLock<HashMap<Uuid, TrackingSession>>>,
    vehicles: Arc<RwLock<HashMap<VehicleId, Vehicle>>>,
    invoices: Arc<RwLock<HashMap<InvoiceId, Invoice>>>,
    payouts: Arc<RwLock<Vec<CourierPayout>>>,
    notifications: Arc<RwLock<HashMap<NotificationId, Notification>>>,
    users: Arc<RwLock<HashMap<UserId, User>>>,
    customers: Arc<RwLock<HashMap<CustomerId, CustomerProfile>>>,
    credentials: Arc<RwLock<HashMap<UserId, Credential>>>,
    refresh_sessions: Arc<RwLock<HashMap<Uuid, RefreshSession>>>,
    tenants: Arc<RwLock<HashMap<String, TenantCompany>>>,
    tenant_memberships: Arc<RwLock<HashMap<(TenantId, UserId), TenantMembership>>>,
    courier_tenants: Arc<RwLock<HashMap<Uuid, TenantId>>>,
    order_tenants: Arc<RwLock<HashMap<OrderId, TenantId>>>,
    vehicle_tenants: Arc<RwLock<HashMap<VehicleId, TenantId>>>,
    proofs_of_delivery: Arc<RwLock<HashMap<OrderId, ProofOfDeliveryRecord>>>,
    webhooks: Arc<RwLock<HashMap<Uuid, WebhookSubscription>>>,
    otp_challenges: Arc<RwLock<HashMap<Uuid, OtpChallenge>>>,
    courier_wallets: Arc<RwLock<HashMap<Uuid, CourierWallet>>>,
    customer_ratings: Arc<RwLock<HashMap<Uuid, CustomerRating>>>,
    support_tickets: Arc<RwLock<HashMap<Uuid, SupportTicket>>>,
    coupons: Arc<RwLock<HashMap<Uuid, Coupon>>>,
    device_push_tokens: Arc<RwLock<HashMap<Uuid, DevicePushToken>>>,
    delivery_pricing: Arc<RwLock<HashMap<TenantId, DeliveryPricing>>>,
    warehouse_hubs: Arc<RwLock<HashMap<Uuid, WarehouseHub>>>,
    hub_manifests: Arc<RwLock<Vec<HubManifestAssignment>>>,
    cold_chain_telemetry: Arc<RwLock<Vec<ColdChainTelemetry>>>,
    field_service_appointments: Arc<RwLock<Vec<FieldServiceAppointment>>>,
    route_breadcrumbs: Arc<RwLock<Vec<RouteBreadcrumb>>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn order_repository(&self) -> InMemoryOrderRepository {
        InMemoryOrderRepository {
            store: Arc::clone(&self.orders),
        }
    }

    pub fn courier_repository(&self) -> InMemoryCourierRepository {
        InMemoryCourierRepository {
            store: Arc::clone(&self.couriers),
        }
    }

    pub fn assignment_repository(&self) -> InMemoryAssignmentRepository {
        InMemoryAssignmentRepository {
            store: Arc::clone(&self.assignments),
        }
    }

    pub fn tracking_repository(&self) -> InMemoryTrackingRepository {
        InMemoryTrackingRepository {
            points: Arc::clone(&self.tracking_points),
            sessions: Arc::clone(&self.tracking_sessions),
        }
    }

    pub fn vehicle_repository(&self) -> InMemoryVehicleRepository {
        InMemoryVehicleRepository {
            store: Arc::clone(&self.vehicles),
        }
    }

    pub fn invoice_repository(&self) -> InMemoryInvoiceRepository {
        InMemoryInvoiceRepository {
            store: Arc::clone(&self.invoices),
        }
    }

    pub fn payout_repository(&self) -> InMemoryPayoutRepository {
        InMemoryPayoutRepository {
            store: Arc::clone(&self.payouts),
        }
    }

    pub fn notification_repository(&self) -> InMemoryNotificationRepository {
        InMemoryNotificationRepository {
            store: Arc::clone(&self.notifications),
        }
    }

    pub fn proof_of_delivery_repository(&self) -> InMemoryProofOfDeliveryRepository {
        InMemoryProofOfDeliveryRepository {
            store: Arc::clone(&self.proofs_of_delivery),
        }
    }

    pub fn webhook_repository(&self) -> InMemoryWebhookRepository {
        InMemoryWebhookRepository {
            store: Arc::clone(&self.webhooks),
        }
    }

    pub fn otp_challenge_repository(&self) -> InMemoryOtpChallengeRepository {
        InMemoryOtpChallengeRepository {
            store: Arc::clone(&self.otp_challenges),
        }
    }

    pub fn courier_wallet_repository(&self) -> InMemoryCourierWalletRepository {
        InMemoryCourierWalletRepository {
            store: Arc::clone(&self.courier_wallets),
        }
    }

    pub fn customer_rating_repository(&self) -> InMemoryCustomerRatingRepository {
        InMemoryCustomerRatingRepository {
            store: Arc::clone(&self.customer_ratings),
        }
    }

    pub fn support_ticket_repository(&self) -> InMemorySupportTicketRepository {
        InMemorySupportTicketRepository {
            store: Arc::clone(&self.support_tickets),
        }
    }

    pub fn coupon_repository(&self) -> InMemoryCouponRepository {
        InMemoryCouponRepository {
            store: Arc::clone(&self.coupons),
        }
    }

    pub fn device_push_token_repository(&self) -> InMemoryDevicePushTokenRepository {
        InMemoryDevicePushTokenRepository {
            store: Arc::clone(&self.device_push_tokens),
        }
    }

    pub fn delivery_pricing_repository(&self) -> InMemoryDeliveryPricingRepository {
        InMemoryDeliveryPricingRepository {
            store: Arc::clone(&self.delivery_pricing),
        }
    }

    pub fn user_repository(&self) -> InMemoryUserRepository {
        InMemoryUserRepository {
            store: Arc::clone(&self.users),
        }
    }

    pub fn customer_repository(&self) -> InMemoryCustomerRepository {
        InMemoryCustomerRepository {
            store: Arc::clone(&self.customers),
        }
    }

    pub fn credential_repository(&self) -> InMemoryCredentialRepository {
        InMemoryCredentialRepository {
            credentials: Arc::clone(&self.credentials),
            sessions: Arc::clone(&self.refresh_sessions),
        }
    }
    pub fn tenant_repository(&self) -> InMemoryTenantRepository {
        InMemoryTenantRepository {
            tenants: Arc::clone(&self.tenants),
            memberships: Arc::clone(&self.tenant_memberships),
            courier_tenants: Arc::clone(&self.courier_tenants),
            order_tenants: Arc::clone(&self.order_tenants),
            vehicle_tenants: Arc::clone(&self.vehicle_tenants),
        }
    }

    pub fn warehouse_hub_repository(&self) -> InMemoryWarehouseHubRepository {
        InMemoryWarehouseHubRepository {
            hubs: Arc::clone(&self.warehouse_hubs),
            manifests: Arc::clone(&self.hub_manifests),
        }
    }

    pub fn cold_chain_telemetry_repository(&self) -> InMemoryColdChainTelemetryRepository {
        InMemoryColdChainTelemetryRepository {
            store: Arc::clone(&self.cold_chain_telemetry),
        }
    }

    pub fn field_service_appointment_repository(
        &self,
    ) -> InMemoryFieldServiceAppointmentRepository {
        InMemoryFieldServiceAppointmentRepository {
            store: Arc::clone(&self.field_service_appointments),
        }
    }

    pub fn route_breadcrumb_repository(&self) -> InMemoryRouteBreadcrumbRepository {
        InMemoryRouteBreadcrumbRepository {
            store: Arc::clone(&self.route_breadcrumbs),
        }
    }
}

#[derive(Clone)]
pub struct InMemoryTenantRepository {
    tenants: Arc<RwLock<HashMap<String, TenantCompany>>>,
    memberships: Arc<RwLock<HashMap<(TenantId, UserId), TenantMembership>>>,
    courier_tenants: Arc<RwLock<HashMap<Uuid, TenantId>>>,
    order_tenants: Arc<RwLock<HashMap<OrderId, TenantId>>>,
    vehicle_tenants: Arc<RwLock<HashMap<VehicleId, TenantId>>>,
}

#[async_trait]
impl TenantRepository for InMemoryTenantRepository {
    async fn create_tenant(&self, tenant: &TenantCompany, slug: &str) -> Result<(), DomainError> {
        let mut tenants = self
            .tenants
            .write()
            .map_err(|_| DomainError::validation("lock poisoned"))?;
        if tenants.contains_key(slug) {
            return Err(DomainError::AlreadyExists(
                "tenant slug already exists".into(),
            ));
        }
        tenants.insert(slug.to_owned(), tenant.clone());
        Ok(())
    }

    async fn has_any_tenant(&self) -> Result<bool, DomainError> {
        Ok(!self
            .tenants
            .read()
            .map_err(|_| DomainError::validation("lock poisoned"))?
            .is_empty())
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<TenantCompany>, DomainError> {
        Ok(self
            .tenants
            .read()
            .map_err(|_| DomainError::validation("lock poisoned"))?
            .get(slug)
            .cloned())
    }

    async fn add_member(&self, membership: &TenantMembership) -> Result<(), DomainError> {
        let mut memberships = self
            .memberships
            .write()
            .map_err(|_| DomainError::validation("lock poisoned"))?;
        let key = (membership.tenant_id, membership.user_id);
        if memberships.contains_key(&key) {
            return Err(DomainError::AlreadyExists(
                "tenant membership already exists".into(),
            ));
        }
        memberships.insert(key, membership.clone());
        Ok(())
    }

    async fn find_membership(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Option<TenantMembership>, DomainError> {
        Ok(self
            .memberships
            .read()
            .map_err(|_| DomainError::validation("lock poisoned"))?
            .get(&(tenant_id, user_id))
            .cloned())
    }
    async fn bind_courier(&self, tenant_id: TenantId, courier_id: Uuid) -> Result<(), DomainError> {
        self.courier_tenants
            .write()
            .map_err(|_| DomainError::validation("lock poisoned"))?
            .insert(courier_id, tenant_id);
        Ok(())
    }
    async fn bind_order(&self, tenant_id: TenantId, order_id: OrderId) -> Result<(), DomainError> {
        self.order_tenants
            .write()
            .map_err(|_| DomainError::validation("lock poisoned"))?
            .insert(order_id, tenant_id);
        Ok(())
    }
    async fn find_courier_tenant(&self, courier_id: Uuid) -> Result<Option<TenantId>, DomainError> {
        Ok(self
            .courier_tenants
            .read()
            .map_err(|_| DomainError::validation("lock poisoned"))?
            .get(&courier_id)
            .copied())
    }
    async fn find_order_tenant(&self, order_id: OrderId) -> Result<Option<TenantId>, DomainError> {
        Ok(self
            .order_tenants
            .read()
            .map_err(|_| DomainError::validation("lock poisoned"))?
            .get(&order_id)
            .copied())
    }
    async fn bind_vehicle(
        &self,
        tenant_id: TenantId,
        vehicle_id: VehicleId,
    ) -> Result<(), DomainError> {
        self.vehicle_tenants
            .write()
            .map_err(|_| DomainError::validation("lock poisoned"))?
            .insert(vehicle_id, tenant_id);
        Ok(())
    }
    async fn find_vehicle_tenant(
        &self,
        vehicle_id: VehicleId,
    ) -> Result<Option<TenantId>, DomainError> {
        Ok(self
            .vehicle_tenants
            .read()
            .map_err(|_| DomainError::validation("lock poisoned"))?
            .get(&vehicle_id)
            .copied())
    }
}

#[derive(Clone)]
pub struct InMemoryCredentialRepository {
    credentials: Arc<RwLock<HashMap<UserId, Credential>>>,
    sessions: Arc<RwLock<HashMap<Uuid, RefreshSession>>>,
}

#[async_trait]
impl CredentialRepository for InMemoryCredentialRepository {
    async fn save_credential(&self, credential: &Credential) -> Result<(), DomainError> {
        self.credentials
            .write()
            .unwrap()
            .insert(credential.user_id, credential.clone());
        Ok(())
    }
    async fn find_credential(&self, user_id: UserId) -> Result<Option<Credential>, DomainError> {
        Ok(self.credentials.read().unwrap().get(&user_id).cloned())
    }
    async fn save_refresh_session(&self, session: &RefreshSession) -> Result<(), DomainError> {
        self.sessions
            .write()
            .unwrap()
            .insert(session.id, session.clone());
        Ok(())
    }
    async fn find_refresh_session(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshSession>, DomainError> {
        Ok(self
            .sessions
            .read()
            .unwrap()
            .values()
            .find(|session| session.token_hash == token_hash)
            .cloned())
    }
    async fn revoke_refresh_session(&self, id: Uuid) -> Result<(), DomainError> {
        let mut sessions = self.sessions.write().unwrap();
        let session = sessions
            .get_mut(&id)
            .ok_or_else(|| DomainError::NotFound("refresh session not found".into()))?;
        session.revoked_at = Some(chrono::Utc::now());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// OrderRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryOrderRepository {
    store: Arc<RwLock<HashMap<OrderId, Order>>>,
}

#[async_trait]
impl OrderRepository for InMemoryOrderRepository {
    async fn create(&self, order: &Order) -> Result<(), DomainError> {
        self.store
            .write()
            .map_err(|_| DomainError::validation("lock poisoned"))?
            .insert(order.id, order.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, DomainError> {
        Ok(self.store.read().unwrap().get(&id).cloned())
    }

    async fn update(&self, order: &Order) -> Result<(), DomainError> {
        self.store.write().unwrap().insert(order.id, order.clone());
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<Order>, DomainError> {
        Ok(self.store.read().unwrap().values().cloned().collect())
    }
}

// ---------------------------------------------------------------------------
// CourierRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryCourierRepository {
    store: Arc<RwLock<HashMap<Uuid, Courier>>>,
}

#[async_trait]
impl CourierRepository for InMemoryCourierRepository {
    async fn create(&self, courier: &Courier) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(courier.id, courier.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Courier>, DomainError> {
        Ok(self.store.read().unwrap().get(&id).cloned())
    }

    async fn list_all(&self) -> Result<Vec<Courier>, DomainError> {
        let mut couriers: Vec<_> = self.store.read().unwrap().values().cloned().collect();
        couriers.sort_by_key(|courier| courier.registered_at);
        Ok(couriers)
    }

    async fn list_available(&self) -> Result<Vec<Courier>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .filter(|courier| courier.status == CourierStatus::Available)
            .cloned()
            .collect())
    }

    async fn update(&self, courier: &Courier) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(courier.id, courier.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// AssignmentRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryAssignmentRepository {
    store: Arc<RwLock<HashMap<OrderId, Assignment>>>,
}

#[async_trait]
impl AssignmentRepository for InMemoryAssignmentRepository {
    async fn create(&self, assignment: &Assignment) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(assignment.order_id, assignment.clone());
        Ok(())
    }

    async fn find_by_order(&self, order_id: OrderId) -> Result<Option<Assignment>, DomainError> {
        Ok(self.store.read().unwrap().get(&order_id).cloned())
    }

    async fn find_pending_offer_for_courier(
        &self,
        courier_id: Uuid,
    ) -> Result<Option<Assignment>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .find(|assignment| {
                assignment.courier_id == courier_id
                    && assignment.status == AssignmentStatus::Offered
            })
            .cloned())
    }

    async fn update(&self, assignment: &Assignment) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(assignment.order_id, assignment.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// TrackingRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryTrackingRepository {
    points: Arc<RwLock<Vec<TrackingPoint>>>,
    sessions: Arc<RwLock<HashMap<Uuid, TrackingSession>>>,
}

#[async_trait]
impl TrackingRepository for InMemoryTrackingRepository {
    async fn record_point(&self, point: &TrackingPoint) -> Result<(), DomainError> {
        self.points.write().unwrap().push(point.clone());
        Ok(())
    }

    async fn create_session(&self, session: &TrackingSession) -> Result<(), DomainError> {
        self.sessions
            .write()
            .unwrap()
            .insert(session.id, session.clone());
        Ok(())
    }

    async fn find_session(&self, id: Uuid) -> Result<Option<TrackingSession>, DomainError> {
        Ok(self.sessions.read().unwrap().get(&id).cloned())
    }

    async fn find_active_session_for_courier(
        &self,
        courier_id: Uuid,
    ) -> Result<Option<TrackingSession>, DomainError> {
        Ok(self
            .sessions
            .read()
            .unwrap()
            .values()
            .find(|s| s.courier_id == courier_id && s.status == TrackingSessionStatus::Active)
            .cloned())
    }

    async fn update_session(&self, session: &TrackingSession) -> Result<(), DomainError> {
        self.sessions
            .write()
            .unwrap()
            .insert(session.id, session.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ProofOfDeliveryRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryProofOfDeliveryRepository {
    store: Arc<RwLock<HashMap<OrderId, ProofOfDeliveryRecord>>>,
}

#[async_trait]
impl ProofOfDeliveryRepository for InMemoryProofOfDeliveryRepository {
    async fn create(&self, proof: &ProofOfDeliveryRecord) -> Result<(), DomainError> {
        let mut store = self.store.write().unwrap();
        if store.contains_key(&OrderId(proof.order_id)) {
            return Err(DomainError::AlreadyExists(
                "proof of delivery already exists for order".into(),
            ));
        }
        store.insert(OrderId(proof.order_id), proof.clone());
        Ok(())
    }

    async fn find_by_order(
        &self,
        order_id: OrderId,
    ) -> Result<Option<ProofOfDeliveryRecord>, DomainError> {
        Ok(self.store.read().unwrap().get(&order_id).cloned())
    }
}

#[derive(Clone)]
pub struct InMemoryWebhookRepository {
    store: Arc<RwLock<HashMap<Uuid, WebhookSubscription>>>,
}

#[async_trait]
impl WebhookRepository for InMemoryWebhookRepository {
    async fn create(&self, subscription: &WebhookSubscription) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(subscription.id, subscription.clone());
        Ok(())
    }
    async fn list_for_tenant(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<WebhookSubscription>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .filter(|item| item.tenant_id == tenant_id)
            .cloned()
            .collect())
    }
    async fn delete(&self, tenant_id: TenantId, id: Uuid) -> Result<(), DomainError> {
        let mut store = self.store.write().unwrap();
        match store.get(&id) {
            Some(item) if item.tenant_id == tenant_id => {
                store.remove(&id);
                Ok(())
            }
            _ => Err(DomainError::NotFound(
                "webhook subscription not found".into(),
            )),
        }
    }
}

// ---------------------------------------------------------------------------
// VehicleRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryVehicleRepository {
    store: Arc<RwLock<HashMap<VehicleId, Vehicle>>>,
}

#[async_trait]
impl VehicleRepository for InMemoryVehicleRepository {
    async fn create(&self, vehicle: &Vehicle) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(vehicle.id, vehicle.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: VehicleId) -> Result<Option<Vehicle>, DomainError> {
        Ok(self.store.read().unwrap().get(&id).cloned())
    }

    async fn find_by_plate(&self, plate: &str) -> Result<Option<Vehicle>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .find(|v| v.plate_number == plate)
            .cloned())
    }

    async fn list_active(&self) -> Result<Vec<Vehicle>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .filter(|v| v.status == VehicleStatus::Active)
            .cloned()
            .collect())
    }

    async fn update(&self, vehicle: &Vehicle) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(vehicle.id, vehicle.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// InvoiceRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryInvoiceRepository {
    store: Arc<RwLock<HashMap<InvoiceId, Invoice>>>,
}

#[async_trait]
impl InvoiceRepository for InMemoryInvoiceRepository {
    async fn create(&self, invoice: &Invoice) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(invoice.id, invoice.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: InvoiceId) -> Result<Option<Invoice>, DomainError> {
        Ok(self.store.read().unwrap().get(&id).cloned())
    }

    async fn find_by_order(&self, order_id: OrderId) -> Result<Option<Invoice>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .find(|inv| inv.order_id == order_id)
            .cloned())
    }

    async fn update(&self, invoice: &Invoice) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(invoice.id, invoice.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CourierPayoutRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryPayoutRepository {
    store: Arc<RwLock<Vec<CourierPayout>>>,
}

#[async_trait]
impl CourierPayoutRepository for InMemoryPayoutRepository {
    async fn create(&self, payout: &CourierPayout) -> Result<(), DomainError> {
        self.store.write().unwrap().push(payout.clone());
        Ok(())
    }

    async fn find_by_courier(&self, courier_id: Uuid) -> Result<Vec<CourierPayout>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .iter()
            .filter(|p| p.courier_id == courier_id)
            .cloned()
            .collect())
    }

    async fn update(&self, payout: &CourierPayout) -> Result<(), DomainError> {
        let mut store = self.store.write().unwrap();
        if let Some(existing) = store.iter_mut().find(|p| p.id == payout.id) {
            *existing = payout.clone();
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// NotificationRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryNotificationRepository {
    store: Arc<RwLock<HashMap<NotificationId, Notification>>>,
}

#[async_trait]
impl NotificationRepository for InMemoryNotificationRepository {
    async fn create(&self, notification: &Notification) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(notification.id, notification.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: NotificationId) -> Result<Option<Notification>, DomainError> {
        Ok(self.store.read().unwrap().get(&id).cloned())
    }

    async fn list_for_recipient(
        &self,
        recipient_id: Uuid,
    ) -> Result<Vec<Notification>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .filter(|n| n.recipient_id == recipient_id)
            .cloned()
            .collect())
    }

    async fn update(&self, notification: &Notification) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(notification.id, notification.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// UserRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryUserRepository {
    store: Arc<RwLock<HashMap<UserId, User>>>,
}

#[async_trait]
impl UserRepository for InMemoryUserRepository {
    async fn create(&self, user: &User) -> Result<(), DomainError> {
        self.store.write().unwrap().insert(user.id, user.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError> {
        Ok(self.store.read().unwrap().get(&id).cloned())
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .find(|u| u.email.eq_ignore_ascii_case(email))
            .cloned())
    }

    async fn find_by_phone(&self, phone: &str) -> Result<Option<User>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .find(|u| u.phone.as_deref() == Some(phone))
            .cloned())
    }

    async fn update(&self, user: &User) -> Result<(), DomainError> {
        self.store.write().unwrap().insert(user.id, user.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// OtpChallengeRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryOtpChallengeRepository {
    store: Arc<RwLock<HashMap<Uuid, OtpChallenge>>>,
}

#[async_trait]
impl OtpChallengeRepository for InMemoryOtpChallengeRepository {
    async fn create(&self, challenge: &OtpChallenge) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(challenge.id, challenge.clone());
        Ok(())
    }

    async fn find_latest_active(
        &self,
        tenant_id: TenantId,
        phone: &str,
        now: DateTime<Utc>,
    ) -> Result<Option<OtpChallenge>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .filter(|challenge| {
                challenge.tenant_id == tenant_id
                    && challenge.phone == phone
                    && !challenge.is_consumed()
                    && !challenge.is_expired(now)
            })
            .max_by_key(|challenge| challenge.created_at)
            .cloned())
    }

    async fn update(&self, challenge: &OtpChallenge) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(challenge.id, challenge.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CourierWalletRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryCourierWalletRepository {
    store: Arc<RwLock<HashMap<Uuid, CourierWallet>>>,
}

#[async_trait]
impl CourierWalletRepository for InMemoryCourierWalletRepository {
    async fn find_by_courier(
        &self,
        courier_id: Uuid,
    ) -> Result<Option<CourierWallet>, DomainError> {
        Ok(self.store.read().unwrap().get(&courier_id).cloned())
    }

    async fn create(&self, wallet: &CourierWallet) -> Result<(), DomainError> {
        let mut store = self.store.write().unwrap();
        if store.contains_key(&wallet.courier_id) {
            return Err(DomainError::AlreadyExists(
                "courier already has a wallet".into(),
            ));
        }
        store.insert(wallet.courier_id, wallet.clone());
        Ok(())
    }

    async fn append_transaction(
        &self,
        wallet: &CourierWallet,
        _transaction: &WalletTransaction,
    ) -> Result<(), DomainError> {
        // The in-memory store keeps the full aggregate (including its
        // transaction history) in one entry, so persisting the post-mutation
        // `wallet` already captures the newly appended transaction.
        self.store
            .write()
            .unwrap()
            .insert(wallet.courier_id, wallet.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CustomerRatingRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryCustomerRatingRepository {
    store: Arc<RwLock<HashMap<Uuid, CustomerRating>>>,
}

#[async_trait]
impl CustomerRatingRepository for InMemoryCustomerRatingRepository {
    async fn create(&self, rating: &CustomerRating) -> Result<(), DomainError> {
        let mut store = self.store.write().unwrap();
        if store
            .values()
            .any(|existing| existing.order_id == rating.order_id)
        {
            return Err(DomainError::AlreadyExists(
                "order already has a rating".into(),
            ));
        }
        store.insert(rating.id, rating.clone());
        Ok(())
    }

    async fn find_by_order(&self, order_id: Uuid) -> Result<Option<CustomerRating>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .find(|rating| rating.order_id == order_id)
            .cloned())
    }

    async fn list_for_courier(&self, courier_id: Uuid) -> Result<Vec<CustomerRating>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .filter(|rating| rating.courier_id == courier_id)
            .cloned()
            .collect())
    }
}

// ---------------------------------------------------------------------------
// SupportTicketRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemorySupportTicketRepository {
    store: Arc<RwLock<HashMap<Uuid, SupportTicket>>>,
}

#[async_trait]
impl SupportTicketRepository for InMemorySupportTicketRepository {
    async fn create(&self, ticket: &SupportTicket) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(ticket.id, ticket.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<SupportTicket>, DomainError> {
        Ok(self.store.read().unwrap().get(&id).cloned())
    }

    async fn list_for_customer(
        &self,
        customer_id: Uuid,
    ) -> Result<Vec<SupportTicket>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .filter(|ticket| ticket.customer_id == customer_id)
            .cloned()
            .collect())
    }

    async fn list_for_tenant(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<SupportTicket>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .filter(|ticket| ticket.tenant_id == tenant_id)
            .cloned()
            .collect())
    }

    async fn update(&self, ticket: &SupportTicket) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(ticket.id, ticket.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CouponRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryCouponRepository {
    store: Arc<RwLock<HashMap<Uuid, Coupon>>>,
}

#[async_trait]
impl CouponRepository for InMemoryCouponRepository {
    async fn create(&self, coupon: &Coupon) -> Result<(), DomainError> {
        let mut store = self.store.write().unwrap();
        let duplicate = store
            .values()
            .any(|existing| existing.tenant_id == coupon.tenant_id && existing.code == coupon.code);
        if duplicate {
            return Err(DomainError::AlreadyExists(
                "a coupon with this code already exists for this tenant".into(),
            ));
        }
        store.insert(coupon.id, coupon.clone());
        Ok(())
    }

    async fn find_by_code(
        &self,
        tenant_id: TenantId,
        code: &str,
    ) -> Result<Option<Coupon>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .find(|coupon| coupon.tenant_id == tenant_id && coupon.code == code)
            .cloned())
    }

    async fn list_for_tenant(&self, tenant_id: TenantId) -> Result<Vec<Coupon>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .filter(|coupon| coupon.tenant_id == tenant_id)
            .cloned()
            .collect())
    }

    async fn update(&self, coupon: &Coupon) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(coupon.id, coupon.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// DeliveryPricingRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryDeliveryPricingRepository {
    store: Arc<RwLock<HashMap<TenantId, DeliveryPricing>>>,
}

#[async_trait]
impl DeliveryPricingRepository for InMemoryDeliveryPricingRepository {
    async fn find_by_tenant(
        &self,
        tenant_id: TenantId,
    ) -> Result<Option<DeliveryPricing>, DomainError> {
        Ok(self.store.read().unwrap().get(&tenant_id).cloned())
    }

    async fn upsert(&self, pricing: &DeliveryPricing) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(pricing.tenant_id, pricing.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// DevicePushTokenRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryDevicePushTokenRepository {
    store: Arc<RwLock<HashMap<Uuid, DevicePushToken>>>,
}

#[async_trait]
impl DevicePushTokenRepository for InMemoryDevicePushTokenRepository {
    async fn create(&self, token: &DevicePushToken) -> Result<(), DomainError> {
        self.store.write().unwrap().insert(token.id, token.clone());
        Ok(())
    }

    async fn find_by_user_and_token(
        &self,
        user_id: UserId,
        device_token: &str,
    ) -> Result<Option<DevicePushToken>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .find(|token| token.user_id == user_id && token.device_token == device_token)
            .cloned())
    }

    async fn list_for_user(&self, user_id: UserId) -> Result<Vec<DevicePushToken>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .filter(|token| token.user_id == user_id)
            .cloned()
            .collect())
    }

    async fn delete(&self, user_id: UserId, id: Uuid) -> Result<(), DomainError> {
        let mut store = self.store.write().unwrap();
        if store.get(&id).is_some_and(|token| token.user_id == user_id) {
            store.remove(&id);
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// CustomerRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryCustomerRepository {
    store: Arc<RwLock<HashMap<CustomerId, CustomerProfile>>>,
}

#[async_trait]
impl CustomerRepository for InMemoryCustomerRepository {
    async fn create(&self, profile: &CustomerProfile) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(profile.id, profile.clone());
        Ok(())
    }

    async fn find_by_id(&self, id: CustomerId) -> Result<Option<CustomerProfile>, DomainError> {
        Ok(self.store.read().unwrap().get(&id).cloned())
    }

    async fn find_by_user(&self, user_id: UserId) -> Result<Option<CustomerProfile>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .values()
            .find(|p| p.user_id == user_id)
            .cloned())
    }

    async fn update(&self, profile: &CustomerProfile) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(profile.id, profile.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// WarehouseHubRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryWarehouseHubRepository {
    hubs: Arc<RwLock<HashMap<Uuid, WarehouseHub>>>,
    manifests: Arc<RwLock<Vec<HubManifestAssignment>>>,
}

#[async_trait]
impl WarehouseHubRepository for InMemoryWarehouseHubRepository {
    async fn create_hub(&self, hub: &WarehouseHub) -> Result<(), DomainError> {
        self.hubs.write().unwrap().insert(hub.id, hub.clone());
        Ok(())
    }

    async fn find_hub_by_id(&self, id: Uuid) -> Result<Option<WarehouseHub>, DomainError> {
        Ok(self.hubs.read().unwrap().get(&id).cloned())
    }

    async fn list_hubs_for_tenant(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<WarehouseHub>, DomainError> {
        Ok(self
            .hubs
            .read()
            .unwrap()
            .values()
            .filter(|hub| hub.tenant_id == tenant_id)
            .cloned()
            .collect())
    }

    async fn update_hub(&self, hub: &WarehouseHub) -> Result<(), DomainError> {
        self.hubs.write().unwrap().insert(hub.id, hub.clone());
        Ok(())
    }

    async fn create_manifest(&self, manifest: &HubManifestAssignment) -> Result<(), DomainError> {
        self.manifests.write().unwrap().push(manifest.clone());
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// ColdChainTelemetryRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryColdChainTelemetryRepository {
    store: Arc<RwLock<Vec<ColdChainTelemetry>>>,
}

#[async_trait]
impl ColdChainTelemetryRepository for InMemoryColdChainTelemetryRepository {
    async fn create(&self, telemetry: &ColdChainTelemetry) -> Result<(), DomainError> {
        self.store.write().unwrap().push(telemetry.clone());
        Ok(())
    }

    async fn list_for_tenant(
        &self,
        tenant_id: TenantId,
        order_id: Option<Uuid>,
    ) -> Result<Vec<ColdChainTelemetry>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .iter()
            .filter(|item| {
                item.tenant_id == tenant_id && order_id.is_none_or(|id| id == item.order_id)
            })
            .cloned()
            .collect())
    }
}

// ---------------------------------------------------------------------------
// FieldServiceAppointmentRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryFieldServiceAppointmentRepository {
    store: Arc<RwLock<Vec<FieldServiceAppointment>>>,
}

#[async_trait]
impl FieldServiceAppointmentRepository for InMemoryFieldServiceAppointmentRepository {
    async fn create(&self, appointment: &FieldServiceAppointment) -> Result<(), DomainError> {
        self.store.write().unwrap().push(appointment.clone());
        Ok(())
    }

    async fn list_for_tenant(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<FieldServiceAppointment>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .iter()
            .filter(|appointment| appointment.tenant_id == tenant_id)
            .cloned()
            .collect())
    }
}

// ---------------------------------------------------------------------------
// RouteBreadcrumbRepository
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct InMemoryRouteBreadcrumbRepository {
    store: Arc<RwLock<Vec<RouteBreadcrumb>>>,
}

#[async_trait]
impl RouteBreadcrumbRepository for InMemoryRouteBreadcrumbRepository {
    async fn create(&self, breadcrumb: &RouteBreadcrumb) -> Result<(), DomainError> {
        self.store.write().unwrap().push(breadcrumb.clone());
        Ok(())
    }

    async fn list_for_courier_and_date(
        &self,
        tenant_id: TenantId,
        courier_id: Uuid,
        date: &str,
    ) -> Result<Vec<RouteBreadcrumb>, DomainError> {
        Ok(self
            .store
            .read()
            .unwrap()
            .iter()
            .filter(|b| {
                b.tenant_id == tenant_id
                    && b.courier_id == courier_id
                    && b.timestamp.date_naive().to_string() == date
            })
            .cloned()
            .collect())
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use qervon_domain::{
        Address, AssignmentStatus, InvoiceStatus, Location, Money, NotificationChannel,
        NotificationStatus, OrderStatus, TrackingSessionStatus, VehicleType,
    };

    fn sample_courier(id: Uuid) -> Courier {
        Courier::create(id, "Test Courier", VehicleType::Car, Utc::now()).expect("courier")
    }

    fn sample_order(id: OrderId) -> Order {
        Order::create(
            id,
            Uuid::now_v7(),
            Address {
                location: Location::new(41.0, 29.0).unwrap(),
                label: Some("pickup".into()),
            },
            Address {
                location: Location::new(41.1, 29.1).unwrap(),
                label: Some("dropoff".into()),
            },
            Money::new(1_000, "TRY").unwrap(),
            Utc::now(),
            None,
            None,
        )
        .expect("order")
    }

    #[tokio::test]
    async fn memory_orders_round_trip() {
        let store = InMemoryStore::new();
        let repo = store.order_repository();
        let order = sample_order(OrderId::new());

        repo.create(&order).await.expect("create");
        let found = repo.find_by_id(order.id).await.expect("find");
        assert_eq!(found, Some(order));
    }

    #[tokio::test]
    async fn memory_couriers_list_only_available() {
        let store = InMemoryStore::new();
        let repo = store.courier_repository();
        let mut busy = sample_courier(Uuid::now_v7());
        busy.go_busy().expect("go busy");

        repo.create(&sample_courier(Uuid::now_v7()))
            .await
            .expect("create");
        repo.create(&busy).await.expect("create");

        let available = repo.list_available().await.expect("list");
        assert_eq!(available.len(), 1);
    }

    #[tokio::test]
    async fn memory_assignments_are_keyed_by_order() {
        let store = InMemoryStore::new();
        let repo = store.assignment_repository();
        let assignment = Assignment::new(OrderId::new(), Uuid::now_v7(), Utc::now()).unwrap();

        repo.create(&assignment).await.expect("create");
        let found = repo.find_by_order(assignment.order_id).await.expect("find");
        assert_eq!(found, Some(assignment));
        assert_eq!(found.unwrap().status, AssignmentStatus::Assigned);
    }

    #[test]
    fn sample_order_starts_pending() {
        assert_eq!(sample_order(OrderId::new()).status, OrderStatus::Pending);
    }

    // ---- New repository tests ----

    #[tokio::test]
    async fn tracking_session_round_trip() {
        let store = InMemoryStore::new();
        let repo = store.tracking_repository();
        let courier_id = Uuid::now_v7();
        let session = TrackingSession::start(courier_id, Utc::now()).unwrap();

        repo.create_session(&session).await.expect("create");
        let found = repo
            .find_active_session_for_courier(courier_id)
            .await
            .expect("find");
        assert!(found.is_some());
        assert_eq!(found.unwrap().status, TrackingSessionStatus::Active);
    }

    #[tokio::test]
    async fn vehicle_find_by_plate() {
        let store = InMemoryStore::new();
        let repo = store.vehicle_repository();
        let vehicle = qervon_domain::Vehicle::register(
            qervon_domain::VehicleId::new(),
            "34 XY 456",
            VehicleType::Motorcycle,
            None,
            Utc::now(),
        )
        .unwrap();

        repo.create(&vehicle).await.expect("create");
        let found = repo.find_by_plate("34 XY 456").await.expect("find");
        assert!(found.is_some());
        assert_eq!(found.unwrap().plate_number, "34 XY 456");
    }

    #[tokio::test]
    async fn invoice_round_trip() {
        let store = InMemoryStore::new();
        let repo = store.invoice_repository();
        let invoice = Invoice::create(
            InvoiceId::new(),
            OrderId::new(),
            Uuid::now_v7(),
            Money::new(5_000, "TRY").unwrap(),
            Utc::now(),
        )
        .unwrap();

        repo.create(&invoice).await.expect("create");
        let found = repo.find_by_id(invoice.id).await.expect("find");
        assert_eq!(found.unwrap().status, InvoiceStatus::Draft);
    }

    #[tokio::test]
    async fn notification_list_for_recipient() {
        let store = InMemoryStore::new();
        let repo = store.notification_repository();
        let recipient = Uuid::now_v7();
        let n1 = Notification::create(
            NotificationId::new(),
            recipient,
            NotificationChannel::Push,
            "Title 1",
            "Body 1",
            Utc::now(),
        )
        .unwrap();
        let n2 = Notification::create(
            NotificationId::new(),
            recipient,
            NotificationChannel::Sms,
            "Title 2",
            "Body 2",
            Utc::now(),
        )
        .unwrap();
        let other = Notification::create(
            NotificationId::new(),
            Uuid::now_v7(), // different recipient
            NotificationChannel::Email,
            "Other",
            "Other body",
            Utc::now(),
        )
        .unwrap();

        repo.create(&n1).await.unwrap();
        repo.create(&n2).await.unwrap();
        repo.create(&other).await.unwrap();

        let list = repo.list_for_recipient(recipient).await.expect("list");
        assert_eq!(list.len(), 2);
        assert!(list.iter().all(|n| n.status == NotificationStatus::Queued));
    }

    #[tokio::test]
    async fn proof_of_delivery_is_saved_once_per_order() {
        let store = InMemoryStore::new();
        let repo = store.proof_of_delivery_repository();
        let order_id = Uuid::now_v7();
        let proof =
            ProofOfDeliveryRecord::new(order_id, Uuid::now_v7(), "Teslim Alan", true, None, None)
                .expect("valid proof");

        repo.create(&proof).await.expect("create proof");
        let found = repo
            .find_by_order(OrderId(order_id))
            .await
            .expect("find proof");
        assert_eq!(found.as_ref().map(|record| record.id), Some(proof.id));
        assert!(repo.create(&proof).await.is_err());
    }

    #[tokio::test]
    async fn user_repository_round_trip() {
        let store = InMemoryStore::new();
        let repo = store.user_repository();
        let user = qervon_domain::User::create(
            qervon_domain::UserId::new(),
            "user@qervon.com",
            "User Name",
            qervon_domain::UserRole::Customer,
            Utc::now(),
        )
        .unwrap();

        repo.create(&user).await.expect("create");
        let found_id = repo.find_by_id(user.id).await.expect("find by id");
        assert_eq!(found_id.as_ref(), Some(&user));

        let found_email = repo
            .find_by_email("USER@qervon.com")
            .await
            .expect("find by email");
        assert_eq!(found_email.as_ref(), Some(&user));
    }

    #[tokio::test]
    async fn customer_repository_round_trip() {
        let store = InMemoryStore::new();
        let repo = store.customer_repository();
        let user_id = qervon_domain::UserId::new();
        let profile = qervon_domain::CustomerProfile::create(
            qervon_domain::CustomerId::new(),
            user_id,
            Utc::now(),
        );

        repo.create(&profile).await.expect("create");
        let found_id = repo.find_by_id(profile.id).await.expect("find by id");
        assert_eq!(found_id.as_ref(), Some(&profile));

        let found_user = repo.find_by_user(user_id).await.expect("find by user");
        assert_eq!(found_user.as_ref(), Some(&profile));
    }
}
