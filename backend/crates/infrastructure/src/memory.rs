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
use qervon_domain::{
    Assignment, AssignmentRepository, Courier, CourierPayout, CourierPayoutRepository,
    CourierRepository, CourierStatus, CustomerId, CustomerProfile, CustomerRepository,
    DomainError, Invoice, InvoiceId, InvoiceRepository, Notification, NotificationId,
    NotificationRepository, Order, OrderId, OrderRepository, TrackingPoint, TrackingRepository,
    TrackingSession, TrackingSessionStatus, User, UserId, UserRepository, Vehicle, VehicleId,
    VehicleRepository, VehicleStatus,
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

    async fn find_by_id(
        &self,
        id: NotificationId,
    ) -> Result<Option<Notification>, DomainError> {
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
        self.store
            .write()
            .unwrap()
            .insert(user.id, user.clone());
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

    async fn update(&self, user: &User) -> Result<(), DomainError> {
        self.store
            .write()
            .unwrap()
            .insert(user.id, user.clone());
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

        let found_email = repo.find_by_email("USER@qervon.com").await.expect("find by email");
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


