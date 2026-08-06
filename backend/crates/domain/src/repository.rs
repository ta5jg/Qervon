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

use crate::courier::Courier;
use crate::dispatch::Assignment;
use crate::error::DomainError;
use crate::order::{Order, OrderId};

#[async_trait]
pub trait OrderRepository: Send + Sync {
    async fn create(&self, order: &Order) -> Result<(), DomainError>;
    async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, DomainError>;
    async fn update(&self, order: &Order) -> Result<(), DomainError>;
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
}

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
