// =============================================================================
// File:           backend/apps/api-gateway/src/state.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Composition root: builds module instances over memory or Postgres storage.
//
// Specification:
//   QAS-000001 through QAS-000006, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use std::sync::Arc;

use qervon_couriers_module::CouriersModule;
use qervon_dispatch_module::DispatchModule;
use qervon_domain::{AssignmentRepository, CourierRepository, OrderRepository};
use qervon_infrastructure::{
    memory::InMemoryStore,
    postgres::{PgAssignmentRepository, PgCourierRepository, PgOrderRepository, PgPoolOptions},
};
use qervon_orders_module::OrdersModule;

type DynOrders = Arc<dyn OrderRepository>;
type DynCouriers = Arc<dyn CourierRepository>;
type DynAssignments = Arc<dyn AssignmentRepository>;

#[derive(Clone)]
pub struct AppState {
    pub orders: Arc<OrdersModule<DynOrders>>,
    pub couriers: Arc<CouriersModule<DynCouriers>>,
    pub dispatch: Arc<DispatchModule<DynOrders, DynCouriers, DynAssignments>>,
}

impl AppState {
    pub async fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let storage = std::env::var("QERVON_STORAGE").unwrap_or_else(|_| "memory".to_string());
        match storage.as_str() {
            "memory" => Ok(Self::memory()),
            "postgres" => Self::postgres().await,
            other => Err(format!("unknown QERVON_STORAGE value: {other}").into()),
        }
    }

    pub fn memory() -> Self {
        let store = InMemoryStore::new();
        Self::with_repositories(
            Arc::new(store.order_repository()),
            Arc::new(store.courier_repository()),
            Arc::new(store.assignment_repository()),
        )
    }

    pub async fn postgres() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL is required when QERVON_STORAGE=postgres")?;
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;
        Ok(Self::with_repositories(
            Arc::new(PgOrderRepository::new(pool.clone())),
            Arc::new(PgCourierRepository::new(pool.clone())),
            Arc::new(PgAssignmentRepository::new(pool)),
        ))
    }

    fn with_repositories(
        orders: DynOrders,
        couriers: DynCouriers,
        assignments: DynAssignments,
    ) -> Self {
        Self {
            orders: Arc::new(OrdersModule::new(orders.clone())),
            couriers: Arc::new(CouriersModule::new(couriers.clone())),
            dispatch: Arc::new(DispatchModule::new(orders, couriers, assignments)),
        }
    }
}
