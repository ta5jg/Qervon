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

use qervon_application::{
    AuthService, CouponService, CourierWalletService, DevicePushService, OtpService,
    PricingService, RatingService, SupportTicketService,
};
use qervon_billing_module::BillingModule;
use qervon_couriers_module::CouriersModule;
use qervon_customers_module::CustomersModule;
use qervon_dispatch_module::DispatchModule;
use qervon_domain::{
    AssignmentRepository, CouponRepository, CourierPayoutRepository, CourierRepository,
    CourierWalletRepository, CredentialRepository, CustomerRatingRepository, CustomerRepository,
    DeliveryPricingRepository, DevicePushTokenRepository, InvoiceRepository,
    NotificationRepository, OrderRepository, OtpChallengeRepository, ProofOfDeliveryRepository,
    RouteBreadcrumb, SupportTicketRepository, TenantRepository, TrackingRepository, UserRepository,
    VehicleRepository, WarehouseHub, WebhookRepository,
};
use qervon_fleet_module::FleetModule;
use qervon_foundation_runtime::{
    FoundationRuntime, ModuleManifest, PolicyDefinition, RuleDefinition, WorkflowDefinition,
    WorkflowTransition,
};
use qervon_identity_module::IdentityModule;
use qervon_infrastructure::{
    memory::InMemoryStore,
    postgres::{
        PgAssignmentRepository, PgCouponRepository, PgCourierPayoutRepository, PgCourierRepository,
        PgCourierWalletRepository, PgCredentialRepository, PgCustomerRatingRepository,
        PgCustomerRepository, PgDeliveryPricingRepository, PgDevicePushTokenRepository,
        PgInvoiceRepository, PgNotificationRepository, PgOrderRepository, PgOtpChallengeRepository,
        PgPoolOptions, PgProofOfDeliveryRepository, PgSupportTicketRepository, PgTenantRepository,
        PgTrackingRepository, PgUserRepository, PgVehicleRepository, PgWebhookRepository,
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
type DynOtpChallenges = Arc<dyn OtpChallengeRepository>;
type DynCourierWallets = Arc<dyn CourierWalletRepository>;
type DynCustomerRatings = Arc<dyn CustomerRatingRepository>;
type DynSupportTickets = Arc<dyn SupportTicketRepository>;
type DynCoupons = Arc<dyn CouponRepository>;
type DynDevicePushTokens = Arc<dyn DevicePushTokenRepository>;
type DynDeliveryPricing = Arc<dyn DeliveryPricingRepository>;

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
    /// AI Fraud Guard flag-and-accept signal for this sample (see
    /// `qervon_domain::TrackingPoint`). Historical points restored from
    /// PostgreSQL carry their persisted value; points restored before this
    /// field existed default to `false`/`0.0`.
    #[serde(default)]
    pub fraud_flagged: bool,
    #[serde(default)]
    pub fraud_risk_score: f64,
}

#[derive(Clone, Serialize, Deserialize)]
struct LocationRelayMessage {
    origin_id: uuid::Uuid,
    event: LocationUpdateEvent,
}

#[derive(Clone)]
pub struct AppState {
    pub foundation: Arc<FoundationRuntime>,
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
    pub otp: Arc<OtpService<DynUsers, DynOtpChallenges>>,
    pub courier_wallets: Arc<CourierWalletService<DynCourierWallets>>,
    pub ratings: Arc<RatingService<DynCustomerRatings, DynOrders>>,
    pub support_tickets: Arc<SupportTicketService<DynSupportTickets>>,
    pub coupons: Arc<CouponService<DynCoupons>>,
    pub device_push: Arc<DevicePushService<DynDevicePushTokens>>,
    pub pricing: Arc<PricingService<DynDeliveryPricing>>,
    pub tenants: DynTenants,
    pub location_tx: tokio::sync::broadcast::Sender<LocationUpdateEvent>,
    pub latest_locations: Arc<std::sync::RwLock<HashMap<uuid::Uuid, LocationUpdateEvent>>>,
    instance_id: uuid::Uuid,
    pub proofs_of_delivery: DynProofsOfDelivery,
    pub webhooks: DynWebhooks,
    /// Production API access token. It is required whenever PostgreSQL storage is used.
    pub api_access_token: Option<Arc<str>>,
    pub token_signing_secret: Option<Arc<str>>,
    /// One-time installation secret required by PostgreSQL-backed first setup.
    pub initial_setup_token: Option<Arc<str>>,
    pub storage_backend: StorageBackend,
    pub started_at: Instant,
    pub runtime_metrics: Arc<ApiRuntimeMetrics>,
    pub postgres_pool: Option<sqlx::PgPool>,
    pub warehouse_hubs: Arc<std::sync::RwLock<Vec<WarehouseHub>>>,
    pub hub_manifests: Arc<std::sync::RwLock<Vec<qervon_domain::HubManifestAssignment>>>,
    pub cold_chain_telemetry: Arc<std::sync::RwLock<Vec<qervon_domain::ColdChainTelemetry>>>,
    pub route_breadcrumbs: Arc<std::sync::RwLock<Vec<RouteBreadcrumb>>>,
    pub field_service_appointments:
        Arc<std::sync::RwLock<Vec<qervon_application::FieldServiceAppointment>>>,
    pub payment_reconciliations: Arc<std::sync::RwLock<Vec<serde_json::Value>>>,
    pub sms_provider_url: Option<String>,
    pub sms_provider_bearer_token: Option<Arc<str>>,
    pub payment_gateway_url: Option<String>,
    pub payment_gateway_bearer_token: Option<Arc<str>>,
    pub push_provider_url: Option<String>,
    pub push_provider_bearer_token: Option<Arc<str>>,
    /// Local-filesystem root for uploaded files (delivery-proof photos
    /// today). Real, working persistence — but not a cloud object store;
    /// see QLS-000013 and BACKEND_BACKLOG.md. Configured via
    /// `QERVON_UPLOADS_DIR`, defaulting to `./data/uploads` for local/dev.
    pub uploads_dir: std::path::PathBuf,
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
        state.initial_setup_token = std::env::var("QERVON_INITIAL_SETUP_TOKEN")
            .ok()
            .filter(|value| value.len() >= 16)
            .map(Arc::<str>::from);
        if state.api_access_token.is_none() && state.token_signing_secret.is_none() {
            return Err("QERVON_API_ACCESS_TOKEN or a 32+ character QERVON_TOKEN_SIGNING_SECRET is required".into());
        }
        if let Ok(dir) = std::env::var("QERVON_UPLOADS_DIR") {
            state.uploads_dir = std::path::PathBuf::from(dir);
        }
        state.sms_provider_url = std::env::var("QERVON_SMS_PROVIDER_URL")
            .ok()
            .filter(|value| !value.trim().is_empty());
        state.sms_provider_bearer_token = std::env::var("QERVON_SMS_PROVIDER_TOKEN")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(Arc::<str>::from);
        state.payment_gateway_url = std::env::var("QERVON_PAYMENT_GATEWAY_URL")
            .ok()
            .filter(|value| !value.trim().is_empty());
        state.payment_gateway_bearer_token = std::env::var("QERVON_PAYMENT_GATEWAY_TOKEN")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(Arc::<str>::from);
        state.push_provider_url = std::env::var("QERVON_PUSH_PROVIDER_URL")
            .ok()
            .filter(|value| !value.trim().is_empty());
        state.push_provider_bearer_token = std::env::var("QERVON_PUSH_PROVIDER_TOKEN")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(Arc::<str>::from);
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
            Arc::new(store.otp_challenge_repository()),
            Arc::new(store.courier_wallet_repository()),
            Arc::new(store.customer_rating_repository()),
            Arc::new(store.support_ticket_repository()),
            Arc::new(store.coupon_repository()),
            Arc::new(store.device_push_token_repository()),
            Arc::new(store.delivery_pricing_repository()),
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
            Arc::new(PgOtpChallengeRepository::new(pool.clone())),
            Arc::new(PgCourierWalletRepository::new(pool.clone())),
            Arc::new(PgCustomerRatingRepository::new(pool.clone())),
            Arc::new(PgSupportTicketRepository::new(pool.clone())),
            Arc::new(PgCouponRepository::new(pool.clone())),
            Arc::new(PgDevicePushTokenRepository::new(pool.clone())),
            Arc::new(PgDeliveryPricingRepository::new(pool.clone())),
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
        otp_challenges: DynOtpChallenges,
        courier_wallets: DynCourierWallets,
        customer_ratings: DynCustomerRatings,
        support_tickets: DynSupportTickets,
        coupons: DynCoupons,
        device_push_tokens: DynDevicePushTokens,
        delivery_pricing: DynDeliveryPricing,
    ) -> Self {
        let (location_tx, _) = tokio::sync::broadcast::channel(100);
        let foundation = build_foundation_runtime();
        Self {
            foundation: Arc::new(foundation),
            orders: Arc::new(OrdersModule::new(orders.clone())),
            couriers: Arc::new(CouriersModule::new(couriers.clone())),
            ratings: Arc::new(RatingService::new(customer_ratings, orders.clone())),
            dispatch: Arc::new(DispatchModule::new(orders, couriers, assignments)),
            tracking: Arc::new(TrackingModule::new(tracking)),
            fleet: Arc::new(FleetModule::new(vehicles)),
            billing: Arc::new(BillingModule::new(invoices, payouts)),
            notifications: Arc::new(NotificationsModule::new(notifications)),
            identity: Arc::new(IdentityModule::new(users.clone())),
            customers: Arc::new(CustomersModule::new(customers)),
            auth: Arc::new(AuthService::new(users.clone(), credentials.clone())),
            otp: Arc::new(OtpService::new(users, otp_challenges)),
            courier_wallets: Arc::new(CourierWalletService::new(courier_wallets)),
            support_tickets: Arc::new(SupportTicketService::new(support_tickets)),
            coupons: Arc::new(CouponService::new(coupons)),
            device_push: Arc::new(DevicePushService::new(device_push_tokens)),
            pricing: Arc::new(PricingService::new(delivery_pricing)),
            credentials,
            tenants,
            location_tx,
            latest_locations: Arc::new(std::sync::RwLock::new(HashMap::new())),
            instance_id: uuid::Uuid::now_v7(),
            proofs_of_delivery,
            webhooks,
            api_access_token: None,
            token_signing_secret: None,
            initial_setup_token: None,
            storage_backend: StorageBackend::Memory,
            started_at: Instant::now(),
            runtime_metrics: Arc::new(ApiRuntimeMetrics::default()),
            postgres_pool: None,
            warehouse_hubs: Arc::new(std::sync::RwLock::new(Vec::new())),
            hub_manifests: Arc::new(std::sync::RwLock::new(Vec::new())),
            cold_chain_telemetry: Arc::new(std::sync::RwLock::new(Vec::new())),
            route_breadcrumbs: Arc::new(std::sync::RwLock::new(Vec::new())),
            field_service_appointments: Arc::new(std::sync::RwLock::new(Vec::new())),
            payment_reconciliations: Arc::new(std::sync::RwLock::new(Vec::new())),
            sms_provider_url: None,
            sms_provider_bearer_token: None,
            payment_gateway_url: None,
            payment_gateway_bearer_token: None,
            push_provider_url: None,
            push_provider_bearer_token: None,
            uploads_dir: std::path::PathBuf::from("./data/uploads"),
        }
    }
}

fn build_foundation_runtime() -> FoundationRuntime {
    let runtime = FoundationRuntime::new();
    for module in [
        ModuleManifest {
            id: "orders".into(),
            version: "1.0.0".into(),
            capabilities: vec!["order.create".into(), "order.lifecycle".into()],
        },
        ModuleManifest {
            id: "dispatch".into(),
            version: "1.0.0".into(),
            capabilities: vec!["assignment.offer".into(), "assignment.accept".into()],
        },
        ModuleManifest {
            id: "tracking".into(),
            version: "1.0.0".into(),
            capabilities: vec!["tracking.publish".into(), "tracking.consume".into()],
        },
        ModuleManifest {
            id: "billing".into(),
            version: "1.0.0".into(),
            capabilities: vec!["invoice.create".into(), "payout.schedule".into()],
        },
    ] {
        runtime.register_module(module);
    }
    runtime.register_rule(RuleDefinition {
        id: "dispatch.max-active-offer-per-courier".into(),
        version: 1,
        expression: "courier.pending_offers <= 1".into(),
    });
    runtime.register_rule(RuleDefinition {
        id: "tracking.min-interval-seconds".into(),
        version: 1,
        expression: "sample_interval_seconds >= 3".into(),
    });
    runtime.register_policy(PolicyDefinition {
        id: "global-default".into(),
        tenant_id: uuid::Uuid::nil(),
        allowed_rule_ids: vec![
            "dispatch.max-active-offer-per-courier".into(),
            "tracking.min-interval-seconds".into(),
        ],
    });
    runtime.register_workflow(WorkflowDefinition {
        id: "delivery-order-lifecycle".into(),
        states: vec![
            "pending".into(),
            "courier_assigned".into(),
            "in_transit".into(),
            "delivered".into(),
        ],
        transitions: vec![
            WorkflowTransition {
                from: "pending".into(),
                to: "courier_assigned".into(),
                event: "assign".into(),
            },
            WorkflowTransition {
                from: "courier_assigned".into(),
                to: "in_transit".into(),
                event: "pickup".into(),
            },
            WorkflowTransition {
                from: "in_transit".into(),
                to: "delivered".into(),
                event: "deliver".into(),
            },
        ],
    });
    runtime.publish_event(
        "foundation.runtime.booted",
        serde_json::json!({
            "source": "api-gateway",
            "status": "ok"
        }),
    );
    runtime
}
