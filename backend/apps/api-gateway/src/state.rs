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
use sqlx::postgres::PgListener;
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use tracing::warn;

use qervon_application::AuthService;
use qervon_billing_module::BillingModule;
use qervon_couriers_module::CouriersModule;
use qervon_customers_module::CustomersModule;
use qervon_dispatch_module::DispatchModule;
use qervon_domain::{
    AssignmentRepository, CourierPayoutRepository, CourierRepository, CredentialRepository,
    CustomerRepository, InvoiceRepository, NotificationRepository, OrderRepository,
    ProofOfDeliveryRepository, TenantRepository, TrackingRepository, UserRepository,
    VehicleRepository, WebhookRepository,
};
use qervon_fleet_module::FleetModule;
use qervon_identity_module::IdentityModule;
use qervon_infrastructure::{
    memory::InMemoryStore,
    postgres::{
        PgAssignmentRepository, PgCourierPayoutRepository, PgCourierRepository,
        PgCredentialRepository, PgCustomerRepository, PgInvoiceRepository,
        PgNotificationRepository, PgOrderRepository, PgPoolOptions, PgProofOfDeliveryRepository,
        PgTenantRepository, PgTrackingRepository, PgUserRepository, PgVehicleRepository,
        PgWebhookRepository,
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
type DynCredentials = Arc<dyn CredentialRepository>;
type DynTenants = Arc<dyn TenantRepository>;
type DynProofsOfDelivery = Arc<dyn ProofOfDeliveryRepository>;
type DynWebhooks = Arc<dyn WebhookRepository>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBackend {
    Memory,
    Postgres,
}

impl StorageBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Memory => "memory",
            Self::Postgres => "postgres",
        }
    }
}

#[derive(Default)]
pub struct ApiRuntimeMetrics {
    responses_2xx: AtomicU64,
    responses_3xx: AtomicU64,
    responses_4xx: AtomicU64,
    responses_5xx: AtomicU64,
    responses_other: AtomicU64,
    duration_microseconds: AtomicU64,
}

impl ApiRuntimeMetrics {
    pub fn observe(&self, status: u16, elapsed: Duration) {
        let counter = match status {
            200..=299 => &self.responses_2xx,
            300..=399 => &self.responses_3xx,
            400..=499 => &self.responses_4xx,
            500..=599 => &self.responses_5xx,
            _ => &self.responses_other,
        };
        counter.fetch_add(1, Ordering::Relaxed);
        let elapsed = u64::try_from(elapsed.as_micros()).unwrap_or(u64::MAX);
        self.duration_microseconds
            .fetch_add(elapsed, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> ApiRuntimeMetricsSnapshot {
        ApiRuntimeMetricsSnapshot {
            responses_2xx: self.responses_2xx.load(Ordering::Relaxed),
            responses_3xx: self.responses_3xx.load(Ordering::Relaxed),
            responses_4xx: self.responses_4xx.load(Ordering::Relaxed),
            responses_5xx: self.responses_5xx.load(Ordering::Relaxed),
            responses_other: self.responses_other.load(Ordering::Relaxed),
            duration_microseconds: self.duration_microseconds.load(Ordering::Relaxed),
        }
    }
}

pub struct ApiRuntimeMetricsSnapshot {
    pub responses_2xx: u64,
    pub responses_3xx: u64,
    pub responses_4xx: u64,
    pub responses_5xx: u64,
    pub responses_other: u64,
    pub duration_microseconds: u64,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct LocationUpdateEvent {
    pub courier_id: uuid::Uuid,
    pub tenant_id: uuid::Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Serialize, Deserialize)]
struct LocationRelayMessage {
    origin_id: uuid::Uuid,
    event: LocationUpdateEvent,
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
    pub credentials: DynCredentials,
    pub auth: Arc<AuthService<DynUsers, DynCredentials>>,
    pub tenants: DynTenants,
    pub location_tx: tokio::sync::broadcast::Sender<LocationUpdateEvent>,
    pub latest_locations: Arc<std::sync::RwLock<HashMap<uuid::Uuid, LocationUpdateEvent>>>,
    instance_id: uuid::Uuid,
    pub proofs_of_delivery: DynProofsOfDelivery,
    pub webhooks: DynWebhooks,
    /// Production API access token. It is required whenever PostgreSQL storage is used.
    pub api_access_token: Option<Arc<str>>,
    pub token_signing_secret: Option<Arc<str>>,
    pub storage_backend: StorageBackend,
    pub started_at: Instant,
    pub runtime_metrics: Arc<ApiRuntimeMetrics>,
    pub postgres_pool: Option<sqlx::PgPool>,
}

impl AppState {
    pub async fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let storage = std::env::var("QERVON_STORAGE").unwrap_or_else(|_| "memory".to_string());
        let storage_backend = match storage.as_str() {
            "memory" => StorageBackend::Memory,
            "postgres" => StorageBackend::Postgres,
            other => return Err(format!("unknown QERVON_STORAGE value: {other}").into()),
        };
        let mut state = match storage_backend {
            StorageBackend::Memory => Self::memory(),
            StorageBackend::Postgres => Self::postgres().await?,
        };
        state.storage_backend = storage_backend;
        state.api_access_token = std::env::var("QERVON_API_ACCESS_TOKEN")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(Arc::<str>::from);
        state.token_signing_secret = std::env::var("QERVON_TOKEN_SIGNING_SECRET")
            .ok()
            .filter(|value| value.len() >= 32)
            .map(Arc::<str>::from);
        if state.api_access_token.is_none() && state.token_signing_secret.is_none() {
            return Err("QERVON_API_ACCESS_TOKEN or a 32+ character QERVON_TOKEN_SIGNING_SECRET is required".into());
        }
        Ok(state)
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
            Arc::new(store.credential_repository()),
            Arc::new(store.tenant_repository()),
            Arc::new(store.proof_of_delivery_repository()),
            Arc::new(store.webhook_repository()),
        )
    }

    pub async fn postgres() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL is required when QERVON_STORAGE=postgres")?;
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&database_url)
            .await?;

        let mut state = Self::with_repositories(
            Arc::new(PgOrderRepository::new(pool.clone())),
            Arc::new(PgCourierRepository::new(pool.clone())),
            Arc::new(PgAssignmentRepository::new(pool.clone())),
            Arc::new(PgTrackingRepository::new(pool.clone())),
            Arc::new(PgVehicleRepository::new(pool.clone())),
            Arc::new(PgInvoiceRepository::new(pool.clone())),
            Arc::new(PgCourierPayoutRepository::new(pool.clone())),
            Arc::new(PgNotificationRepository::new(pool.clone())),
            Arc::new(PgUserRepository::new(pool.clone())),
            Arc::new(PgCustomerRepository::new(pool.clone())),
            Arc::new(PgCredentialRepository::new(pool.clone())),
            Arc::new(PgTenantRepository::new(pool.clone())),
            Arc::new(PgProofOfDeliveryRepository::new(pool.clone())),
            Arc::new(PgWebhookRepository::new(pool.clone())),
        );
        state.postgres_pool = Some(pool);
        state.start_location_relay(database_url);
        Ok(state)
    }

    /// Publishes the location locally and, in PostgreSQL mode, relays it to
    /// every API instance through the database notification channel.
    pub async fn publish_location(&self, event: LocationUpdateEvent) -> Result<(), sqlx::Error> {
        self.accept_location(event.clone());
        if let Some(pool) = &self.postgres_pool {
            let payload = serde_json::to_string(&LocationRelayMessage {
                origin_id: self.instance_id,
                event,
            })
            .expect("location relay message is serializable");
            sqlx::query("SELECT pg_notify('qervon_location_updates', $1)")
                .bind(payload)
                .execute(pool)
                .await?;
        }
        Ok(())
    }

    fn accept_location(&self, event: LocationUpdateEvent) {
        if let Ok(mut locations) = self.latest_locations.write() {
            locations.insert(event.courier_id, event.clone());
        }
        let _ = self.location_tx.send(event);
    }

    fn start_location_relay(&self, database_url: String) {
        let locations = Arc::clone(&self.latest_locations);
        let tx = self.location_tx.clone();
        let instance_id = self.instance_id;
        tokio::spawn(async move {
            loop {
                let mut listener = match PgListener::connect(&database_url).await {
                    Ok(listener) => listener,
                    Err(error) => {
                        warn!(error = %error, "location relay connection failed; retrying");
                        sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                };
                if let Err(error) = listener.listen("qervon_location_updates").await {
                    warn!(error = %error, "location relay subscription failed; retrying");
                    sleep(Duration::from_secs(2)).await;
                    continue;
                }
                loop {
                    let notification = match listener.recv().await {
                        Ok(notification) => notification,
                        Err(error) => {
                            warn!(error = %error, "location relay receive failed; reconnecting");
                            break;
                        }
                    };
                    let Ok(message) =
                        serde_json::from_str::<LocationRelayMessage>(notification.payload())
                    else {
                        warn!("discarding malformed location relay message");
                        continue;
                    };
                    if message.origin_id == instance_id {
                        continue;
                    }
                    if let Ok(mut current) = locations.write() {
                        current.insert(message.event.courier_id, message.event.clone());
                    }
                    let _ = tx.send(message.event);
                }
                sleep(Duration::from_secs(1)).await;
            }
        });
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
        credentials: DynCredentials,
        tenants: DynTenants,
        proofs_of_delivery: DynProofsOfDelivery,
        webhooks: DynWebhooks,
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
            identity: Arc::new(IdentityModule::new(users.clone())),
            customers: Arc::new(CustomersModule::new(customers)),
            auth: Arc::new(AuthService::new(users, credentials.clone())),
            credentials,
            tenants,
            location_tx,
            latest_locations: Arc::new(std::sync::RwLock::new(HashMap::new())),
            instance_id: uuid::Uuid::now_v7(),
            proofs_of_delivery,
            webhooks,
            api_access_token: None,
            token_signing_secret: None,
            storage_backend: StorageBackend::Memory,
            started_at: Instant::now(),
            runtime_metrics: Arc::new(ApiRuntimeMetrics::default()),
            postgres_pool: None,
        }
    }
}
