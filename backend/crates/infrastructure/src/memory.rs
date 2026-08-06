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
    Assignment, AssignmentRepository, Courier, CourierRepository, CourierStatus, DomainError,
    Order, OrderId, OrderRepository,
};
use uuid::Uuid;

#[derive(Clone, Default)]
pub struct InMemoryStore {
    orders: Arc<RwLock<HashMap<OrderId, Order>>>,
    couriers: Arc<RwLock<HashMap<Uuid, Courier>>>,
    assignments: Arc<RwLock<HashMap<OrderId, Assignment>>>,
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
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use qervon_domain::{Address, AssignmentStatus, Location, Money, OrderStatus, VehicleType};

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
}
