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

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use qervon_billing_module::BillingModule;
use qervon_couriers_module::CouriersModule;
use qervon_customers_module::CustomersModule;
use qervon_dispatch_module::DispatchModule;
use qervon_domain::{
    AssignmentRepository, CourierPayoutRepository, CourierRepository, CustomerRepository,
    InvoiceRepository, NotificationRepository, OrderRepository, TrackingRepository,
    UserRepository, VehicleRepository,
};
use qervon_fleet_module::FleetModule;
use qervon_identity_module::IdentityModule;
use qervon_infrastructure::{
    memory::InMemoryStore,
    postgres::{
        PgAssignmentRepository, PgCourierRepository, PgOrderRepository, PgPoolOptions,
        PgUserRepository,
    },
};
use qervon_notifications_module::NotificationsModule;
use qervon_orders_module::OrdersModule;
use qervon_tracking_module::TrackingModule;

type DynOrders = Arc<dyn OrderRepository>;
type DynCouriers = Arc<dyn CourierRepository>;
type DynAssignments = Arc<dyn AssignmentRepository>;
type DynTracking = Arc<dyn TrackingRepository>;
type DynVehicles = Arc<dyn VehicleRepository>;
type DynInvoices = Arc<dyn InvoiceRepository>;
type DynPayouts = Arc<dyn CourierPayoutRepository>;
type DynNotifications = Arc<dyn NotificationRepository>;
type DynUsers = Arc<dyn UserRepository>;
type DynCustomers = Arc<dyn CustomerRepository>;

#[derive(Clone, Serialize, Deserialize)]
pub struct LocationUpdateEvent {
    pub courier_id: uuid::Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone)]
pub struct AppState {
    pub orders: Arc<OrdersModule<DynOrders>>,
    pub couriers: Arc<CouriersModule<DynCouriers>>,
    pub dispatch: Arc<DispatchModule<DynOrders, DynCouriers, DynAssignments>>,
    pub tracking: Arc<TrackingModule<DynTracking>>,
    pub fleet: Arc<FleetModule<DynVehicles>>,
    pub billing: Arc<BillingModule<DynInvoices, DynPayouts>>,
    pub notifications: Arc<NotificationsModule<DynNotifications>>,
    pub identity: Arc<IdentityModule<DynUsers>>,
    pub customers: Arc<CustomersModule<DynCustomers>>,
    pub location_tx: tokio::sync::broadcast::Sender<LocationUpdateEvent>,
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
            Arc::new(store.tracking_repository()),
            Arc::new(store.vehicle_repository()),
            Arc::new(store.invoice_repository()),
            Arc::new(store.payout_repository()),
            Arc::new(store.notification_repository()),
            Arc::new(store.user_repository()),
            Arc::new(store.customer_repository()),
        )
    }

    pub async fn postgres() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL is required when QERVON_STORAGE=postgres")?;
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;
        
        let store = InMemoryStore::new();
        Ok(Self::with_repositories(
            Arc::new(PgOrderRepository::new(pool.clone())),
            Arc::new(PgCourierRepository::new(pool.clone())),
            Arc::new(PgAssignmentRepository::new(pool.clone())),
            Arc::new(store.tracking_repository()),
            Arc::new(store.vehicle_repository()),
            Arc::new(store.invoice_repository()),
            Arc::new(store.payout_repository()),
            Arc::new(store.notification_repository()),
            Arc::new(PgUserRepository::new(pool)),
            Arc::new(store.customer_repository()),
        ))
    }

    #[allow(clippy::too_many_arguments)]
    fn with_repositories(
        orders: DynOrders,
        couriers: DynCouriers,
        assignments: DynAssignments,
        tracking: DynTracking,
        vehicles: DynVehicles,
        invoices: DynInvoices,
        payouts: DynPayouts,
        notifications: DynNotifications,
        users: DynUsers,
        customers: DynCustomers,
    ) -> Self {
        let (location_tx, _) = tokio::sync::broadcast::channel(100);
        Self {
            orders: Arc::new(OrdersModule::new(orders.clone())),
            couriers: Arc::new(CouriersModule::new(couriers.clone())),
            dispatch: Arc::new(DispatchModule::new(orders, couriers, assignments)),
            tracking: Arc::new(TrackingModule::new(tracking)),
            fleet: Arc::new(FleetModule::new(vehicles)),
            billing: Arc::new(BillingModule::new(invoices, payouts)),
            notifications: Arc::new(NotificationsModule::new(notifications)),
            identity: Arc::new(IdentityModule::new(users)),
            customers: Arc::new(CustomersModule::new(customers)),
            location_tx,
        }
    }
}
