// =============================================================================
// File:           backend/crates/infrastructure/src/postgres.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   PostgreSQL repository adapters implementing the domain repository ports.
//
// Specification:
//   QAS-000002, QAS-000005, QAS-000006, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use async_trait::async_trait;
use chrono::{DateTime, NaiveDate, Utc};
use qervon_domain::{
    Address, Assignment, AssignmentRepository, Courier, CourierPayout, CourierPayoutRepository,
    CourierRepository, Credential, CredentialRepository, CustomerId, CustomerProfile,
    CustomerRepository, DomainError, Invoice, InvoiceId, InvoiceRepository, InvoiceStatus,
    Location, Money, Notification, NotificationChannel, NotificationId, NotificationRepository,
    NotificationStatus, Order, OrderId, OrderRepository, PayoutStatus, ProofOfDeliveryRecord,
    ProofOfDeliveryRepository, RefreshSession, SavedAddress, TenantCompany, TenantId,
    TenantMemberRole, TenantMembership, TenantRepository, TrackingPoint, TrackingRepository,
    TrackingSession, TrackingSessionStatus, User, UserId, UserRepository, Vehicle, VehicleId,
    VehicleRepository, VehicleStatus, WebhookRepository, WebhookSubscription,
};
use sqlx::{FromRow, PgPool, Postgres, Transaction};
use uuid::Uuid;

fn map_db_error(error: sqlx::Error) -> DomainError {
    if let sqlx::Error::Database(db) = &error {
        if db.code().as_deref() == Some("23505") {
            return DomainError::AlreadyExists("record already exists".to_string());
        }
        if db.code().as_deref() == Some("23503") {
            return DomainError::Validation("referenced record does not exist".to_string());
        }
    }
    DomainError::validation(format!("database error: {error}"))
}

fn map_row_absent() -> DomainError {
    DomainError::NotFound("row not found".to_string())
}

#[derive(FromRow)]
struct OrderRow {
    id: Uuid,
    customer_id: Uuid,
    pickup_lat: f64,
    pickup_lon: f64,
    pickup_label: Option<String>,
    dropoff_lat: f64,
    dropoff_lon: f64,
    dropoff_label: Option<String>,
    status: String,
    fare_amount_minor: i64,
    fare_currency: String,
    assigned_courier_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    delivered_at: Option<DateTime<Utc>>,
}

impl OrderRow {
    fn into_domain(self) -> Result<Order, DomainError> {
        Ok(Order {
            id: OrderId(self.id),
            customer_id: self.customer_id,
            pickup: Address {
                location: Location::new(self.pickup_lat, self.pickup_lon)?,
                label: self.pickup_label,
            },
            dropoff: Address {
                location: Location::new(self.dropoff_lat, self.dropoff_lon)?,
                label: self.dropoff_label,
            },
            status: self.status.parse()?,
            fare: Money::new(self.fare_amount_minor, self.fare_currency)?,
            assigned_courier_id: self.assigned_courier_id,
            created_at: self.created_at,
            delivered_at: self.delivered_at,
        })
    }
}

#[derive(Clone)]
pub struct PgOrderRepository {
    pool: PgPool,
}

impl PgOrderRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const ORDER_COLUMNS: &str = "id, customer_id, pickup_lat, pickup_lon, pickup_label, \
    dropoff_lat, dropoff_lon, dropoff_label, status, fare_amount_minor, fare_currency, \
    assigned_courier_id, created_at, delivered_at";

#[async_trait]
impl OrderRepository for PgOrderRepository {
    async fn create(&self, order: &Order) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO orders.orders (id, customer_id, pickup_lat, pickup_lon, pickup_label, \
             dropoff_lat, dropoff_lon, dropoff_label, status, fare_amount_minor, fare_currency, \
             assigned_courier_id, created_at, delivered_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)",
        )
        .bind(order.id.0)
        .bind(order.customer_id)
        .bind(order.pickup.location.latitude)
        .bind(order.pickup.location.longitude)
        .bind(&order.pickup.label)
        .bind(order.dropoff.location.latitude)
        .bind(order.dropoff.location.longitude)
        .bind(&order.dropoff.label)
        .bind(order.status.as_str())
        .bind(order.fare.amount_minor)
        .bind(&order.fare.currency)
        .bind(order.assigned_courier_id)
        .bind(order.created_at)
        .bind(order.delivered_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: OrderId) -> Result<Option<Order>, DomainError> {
        let row: Option<OrderRow> = sqlx::query_as(&format!(
            "SELECT {ORDER_COLUMNS} FROM orders.orders WHERE id = $1"
        ))
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        row.map(OrderRow::into_domain).transpose()
    }

    async fn update(&self, order: &Order) -> Result<(), DomainError> {
        let affected = sqlx::query(
            "UPDATE orders.orders SET status = $2, assigned_courier_id = $3, delivered_at = $4 \
             WHERE id = $1",
        )
        .bind(order.id.0)
        .bind(order.status.as_str())
        .bind(order.assigned_courier_id)
        .bind(order.delivered_at)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if affected == 0 {
            return Err(map_row_absent());
        }
        Ok(())
    }

    async fn list_all(&self) -> Result<Vec<Order>, DomainError> {
        let rows: Vec<OrderRow> = sqlx::query_as(&format!(
            "SELECT {ORDER_COLUMNS} FROM orders.orders ORDER BY created_at DESC"
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;
        rows.into_iter().map(OrderRow::into_domain).collect()
    }
}

#[derive(Clone)]
pub struct PgProofOfDeliveryRepository {
    pool: PgPool,
}

impl PgProofOfDeliveryRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct ProofOfDeliveryRow {
    id: Uuid,
    order_id: Uuid,
    courier_id: Uuid,
    recipient_name: String,
    qr_barcode_verified: bool,
    digital_signature_base64: Option<String>,
    photo_evidence_url: Option<String>,
    delivered_at: DateTime<Utc>,
}

impl From<ProofOfDeliveryRow> for ProofOfDeliveryRecord {
    fn from(row: ProofOfDeliveryRow) -> Self {
        Self {
            id: row.id,
            order_id: row.order_id,
            courier_id: row.courier_id,
            recipient_name: row.recipient_name,
            qr_barcode_verified: row.qr_barcode_verified,
            digital_signature_base64: row.digital_signature_base64,
            photo_evidence_url: row.photo_evidence_url,
            delivered_at: row.delivered_at,
        }
    }
}

#[async_trait]
impl ProofOfDeliveryRepository for PgProofOfDeliveryRepository {
    async fn create(&self, proof: &ProofOfDeliveryRecord) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO delivery.proofs_of_delivery \
             (id, order_id, courier_id, recipient_name, qr_barcode_verified, \
             digital_signature_base64, photo_evidence_url, delivered_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(proof.id)
        .bind(proof.order_id)
        .bind(proof.courier_id)
        .bind(&proof.recipient_name)
        .bind(proof.qr_barcode_verified)
        .bind(&proof.digital_signature_base64)
        .bind(&proof.photo_evidence_url)
        .bind(proof.delivered_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    async fn find_by_order(
        &self,
        order_id: OrderId,
    ) -> Result<Option<ProofOfDeliveryRecord>, DomainError> {
        let row: Option<ProofOfDeliveryRow> = sqlx::query_as(
            "SELECT id, order_id, courier_id, recipient_name, qr_barcode_verified, \
             digital_signature_base64, photo_evidence_url, delivered_at \
             FROM delivery.proofs_of_delivery WHERE order_id = $1",
        )
        .bind(order_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(row.map(Into::into))
    }
}

#[derive(Clone)]
pub struct PgInvoiceRepository {
    pool: PgPool,
}

impl PgInvoiceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct InvoiceRow {
    id: Uuid,
    order_id: Uuid,
    customer_id: Uuid,
    amount_minor: i64,
    currency: String,
    status: String,
    created_at: DateTime<Utc>,
    issued_at: Option<DateTime<Utc>>,
    paid_at: Option<DateTime<Utc>>,
}

impl InvoiceRow {
    fn into_domain(self) -> Result<Invoice, DomainError> {
        Ok(Invoice {
            id: InvoiceId(self.id),
            order_id: OrderId(self.order_id),
            customer_id: self.customer_id,
            amount: Money::new(self.amount_minor, self.currency)?,
            status: self.status.parse::<InvoiceStatus>()?,
            created_at: self.created_at,
            issued_at: self.issued_at,
            paid_at: self.paid_at,
        })
    }
}

#[async_trait]
impl InvoiceRepository for PgInvoiceRepository {
    async fn create(&self, invoice: &Invoice) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO billing.delivery_invoices \
             (id, order_id, customer_id, amount_minor, currency, status, created_at, issued_at, paid_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)",
        )
        .bind(invoice.id.0)
        .bind(invoice.order_id.0)
        .bind(invoice.customer_id)
        .bind(invoice.amount.amount_minor)
        .bind(&invoice.amount.currency)
        .bind(invoice.status.as_str())
        .bind(invoice.created_at)
        .bind(invoice.issued_at)
        .bind(invoice.paid_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: InvoiceId) -> Result<Option<Invoice>, DomainError> {
        let row: Option<InvoiceRow> = sqlx::query_as(
            "SELECT id, order_id, customer_id, amount_minor, currency, status, created_at, issued_at, paid_at \
             FROM billing.delivery_invoices WHERE id = $1",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        row.map(InvoiceRow::into_domain).transpose()
    }

    async fn find_by_order(&self, order_id: OrderId) -> Result<Option<Invoice>, DomainError> {
        let row: Option<InvoiceRow> = sqlx::query_as(
            "SELECT id, order_id, customer_id, amount_minor, currency, status, created_at, issued_at, paid_at \
             FROM billing.delivery_invoices WHERE order_id = $1",
        )
        .bind(order_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        row.map(InvoiceRow::into_domain).transpose()
    }

    async fn update(&self, invoice: &Invoice) -> Result<(), DomainError> {
        let affected = sqlx::query(
            "UPDATE billing.delivery_invoices \
             SET status = $2, issued_at = $3, paid_at = $4 WHERE id = $1",
        )
        .bind(invoice.id.0)
        .bind(invoice.status.as_str())
        .bind(invoice.issued_at)
        .bind(invoice.paid_at)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if affected == 0 {
            return Err(map_row_absent());
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct PgCourierPayoutRepository {
    pool: PgPool,
}

impl PgCourierPayoutRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct CourierPayoutRow {
    id: Uuid,
    courier_id: Uuid,
    period_start: DateTime<Utc>,
    period_end: DateTime<Utc>,
    gross_amount_minor: i64,
    commission_amount_minor: i64,
    net_amount_minor: i64,
    currency: String,
    status: String,
    created_at: DateTime<Utc>,
}

impl CourierPayoutRow {
    fn into_domain(self) -> Result<CourierPayout, DomainError> {
        Ok(CourierPayout {
            id: self.id,
            courier_id: self.courier_id,
            period_start: self.period_start,
            period_end: self.period_end,
            gross_amount: Money::new(self.gross_amount_minor, self.currency.clone())?,
            commission: Money::new(self.commission_amount_minor, self.currency.clone())?,
            net_amount: Money::new(self.net_amount_minor, self.currency)?,
            status: self.status.parse::<PayoutStatus>()?,
            created_at: self.created_at,
        })
    }
}

#[async_trait]
impl CourierPayoutRepository for PgCourierPayoutRepository {
    async fn create(&self, payout: &CourierPayout) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO billing.courier_payouts \
             (id, courier_id, period_start, period_end, gross_amount_minor, \
              commission_amount_minor, net_amount_minor, currency, status, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(payout.id)
        .bind(payout.courier_id)
        .bind(payout.period_start)
        .bind(payout.period_end)
        .bind(payout.gross_amount.amount_minor)
        .bind(payout.commission.amount_minor)
        .bind(payout.net_amount.amount_minor)
        .bind(&payout.gross_amount.currency)
        .bind(payout.status.as_str())
        .bind(payout.created_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    async fn find_by_courier(&self, courier_id: Uuid) -> Result<Vec<CourierPayout>, DomainError> {
        let rows: Vec<CourierPayoutRow> = sqlx::query_as(
            "SELECT id, courier_id, period_start, period_end, gross_amount_minor, \
             commission_amount_minor, net_amount_minor, currency, status, created_at \
             FROM billing.courier_payouts WHERE courier_id = $1 ORDER BY period_start DESC",
        )
        .bind(courier_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;
        rows.into_iter()
            .map(CourierPayoutRow::into_domain)
            .collect()
    }

    async fn update(&self, payout: &CourierPayout) -> Result<(), DomainError> {
        let affected = sqlx::query(
            "UPDATE billing.courier_payouts \
             SET period_start = $2, period_end = $3, gross_amount_minor = $4, \
                 commission_amount_minor = $5, net_amount_minor = $6, currency = $7, status = $8 \
             WHERE id = $1",
        )
        .bind(payout.id)
        .bind(payout.period_start)
        .bind(payout.period_end)
        .bind(payout.gross_amount.amount_minor)
        .bind(payout.commission.amount_minor)
        .bind(payout.net_amount.amount_minor)
        .bind(&payout.gross_amount.currency)
        .bind(payout.status.as_str())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if affected == 0 {
            return Err(map_row_absent());
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct PgVehicleRepository {
    pool: PgPool,
}

impl PgVehicleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct VehicleRow {
    id: Uuid,
    courier_id: Option<Uuid>,
    kind: String,
    plate: Option<String>,
    status: String,
    insurance_expiry: Option<NaiveDate>,
    created_at: DateTime<Utc>,
}

impl VehicleRow {
    fn into_domain(self) -> Result<Vehicle, DomainError> {
        let plate_number = self
            .plate
            .filter(|plate| !plate.trim().is_empty())
            .ok_or_else(|| DomainError::validation("vehicle plate is missing"))?;
        let status = match self.status.as_str() {
            "operational" => VehicleStatus::Active,
            "maintenance" => VehicleStatus::Maintenance,
            "retired" => VehicleStatus::Decommissioned,
            other => {
                return Err(DomainError::validation(format!(
                    "unknown vehicle status: {other}"
                )))
            }
        };
        Ok(Vehicle {
            id: VehicleId(self.id),
            plate_number,
            vehicle_type: self.kind.parse()?,
            status,
            assigned_courier_id: self.courier_id,
            insurance_expiry: self.insurance_expiry,
            registered_at: self.created_at,
        })
    }
}

const VEHICLE_COLUMNS: &str = "id, courier_id, kind, plate, status, insurance_expiry, created_at";

fn db_vehicle_status(status: VehicleStatus) -> &'static str {
    match status {
        VehicleStatus::Active => "operational",
        VehicleStatus::Maintenance => "maintenance",
        VehicleStatus::Decommissioned => "retired",
    }
}

#[async_trait]
impl VehicleRepository for PgVehicleRepository {
    async fn create(&self, vehicle: &Vehicle) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO fleet.vehicles \
             (id, courier_id, kind, plate, status, insurance_expiry, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(vehicle.id.0)
        .bind(vehicle.assigned_courier_id)
        .bind(vehicle.vehicle_type.as_str())
        .bind(&vehicle.plate_number)
        .bind(db_vehicle_status(vehicle.status))
        .bind(vehicle.insurance_expiry)
        .bind(vehicle.registered_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: VehicleId) -> Result<Option<Vehicle>, DomainError> {
        let row: Option<VehicleRow> = sqlx::query_as(&format!(
            "SELECT {VEHICLE_COLUMNS} FROM fleet.vehicles WHERE id = $1"
        ))
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        row.map(VehicleRow::into_domain).transpose()
    }

    async fn find_by_plate(&self, plate: &str) -> Result<Option<Vehicle>, DomainError> {
        let row: Option<VehicleRow> = sqlx::query_as(&format!(
            "SELECT {VEHICLE_COLUMNS} FROM fleet.vehicles WHERE lower(plate) = lower($1)"
        ))
        .bind(plate)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        row.map(VehicleRow::into_domain).transpose()
    }

    async fn list_active(&self) -> Result<Vec<Vehicle>, DomainError> {
        let rows: Vec<VehicleRow> = sqlx::query_as(&format!(
            "SELECT {VEHICLE_COLUMNS} FROM fleet.vehicles WHERE status = 'operational' ORDER BY created_at ASC"
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;
        rows.into_iter().map(VehicleRow::into_domain).collect()
    }

    async fn update(&self, vehicle: &Vehicle) -> Result<(), DomainError> {
        let affected = sqlx::query(
            "UPDATE fleet.vehicles \
             SET courier_id = $2, kind = $3, plate = $4, status = $5, insurance_expiry = $6 \
             WHERE id = $1",
        )
        .bind(vehicle.id.0)
        .bind(vehicle.assigned_courier_id)
        .bind(vehicle.vehicle_type.as_str())
        .bind(&vehicle.plate_number)
        .bind(db_vehicle_status(vehicle.status))
        .bind(vehicle.insurance_expiry)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if affected == 0 {
            return Err(map_row_absent());
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct PgCustomerRepository {
    pool: PgPool,
}

impl PgCustomerRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct CustomerProfileRow {
    id: Uuid,
    user_id: Uuid,
    company_name: Option<String>,
    tax_id: Option<String>,
    loyalty_points: i64,
    created_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct CustomerAddressRow {
    id: Uuid,
    label: String,
    latitude: f64,
    longitude: f64,
    full_address: String,
    is_default: bool,
}

impl TryFrom<CustomerAddressRow> for SavedAddress {
    type Error = DomainError;

    fn try_from(row: CustomerAddressRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: row.id,
            label: row.label,
            location: Location::new(row.latitude, row.longitude)?,
            full_address: row.full_address,
            is_default: row.is_default,
        })
    }
}

async fn load_customer_addresses(
    pool: &PgPool,
    customer_id: CustomerId,
) -> Result<Vec<SavedAddress>, DomainError> {
    let rows: Vec<CustomerAddressRow> = sqlx::query_as(
        "SELECT id, label, latitude, longitude, full_address, is_default \
         FROM identity.customer_addresses WHERE customer_profile_id = $1 \
         ORDER BY is_default DESC, id ASC",
    )
    .bind(customer_id.0)
    .fetch_all(pool)
    .await
    .map_err(map_db_error)?;
    rows.into_iter().map(SavedAddress::try_from).collect()
}

async fn insert_customer_addresses(
    transaction: &mut Transaction<'_, Postgres>,
    profile: &CustomerProfile,
) -> Result<(), DomainError> {
    for address in &profile.addresses {
        sqlx::query(
            "INSERT INTO identity.customer_addresses \
             (id, customer_profile_id, label, latitude, longitude, full_address, is_default) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(address.id)
        .bind(profile.id.0)
        .bind(&address.label)
        .bind(address.location.latitude)
        .bind(address.location.longitude)
        .bind(&address.full_address)
        .bind(address.is_default)
        .execute(&mut **transaction)
        .await
        .map_err(map_db_error)?;
    }
    Ok(())
}

async fn hydrate_customer_profile(
    pool: &PgPool,
    row: CustomerProfileRow,
) -> Result<CustomerProfile, DomainError> {
    let id = CustomerId(row.id);
    Ok(CustomerProfile {
        id,
        user_id: UserId(row.user_id),
        company_name: row.company_name,
        tax_id: row.tax_id,
        addresses: load_customer_addresses(pool, id).await?,
        loyalty_points: u64::try_from(row.loyalty_points)
            .map_err(|_| DomainError::validation("customer loyalty points cannot be negative"))?,
        created_at: row.created_at,
    })
}

#[async_trait]
impl CustomerRepository for PgCustomerRepository {
    async fn create(&self, profile: &CustomerProfile) -> Result<(), DomainError> {
        let mut transaction = self.pool.begin().await.map_err(map_db_error)?;
        sqlx::query(
            "INSERT INTO identity.customer_profiles \
             (id, user_id, company_name, tax_id, loyalty_points, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(profile.id.0)
        .bind(profile.user_id.0)
        .bind(&profile.company_name)
        .bind(&profile.tax_id)
        .bind(i64::try_from(profile.loyalty_points).map_err(|_| {
            DomainError::validation("customer loyalty points exceed PostgreSQL bigint range")
        })?)
        .bind(profile.created_at)
        .execute(&mut *transaction)
        .await
        .map_err(map_db_error)?;
        insert_customer_addresses(&mut transaction, profile).await?;
        transaction.commit().await.map_err(map_db_error)
    }

    async fn find_by_id(&self, id: CustomerId) -> Result<Option<CustomerProfile>, DomainError> {
        let row: Option<CustomerProfileRow> = sqlx::query_as(
            "SELECT id, user_id, company_name, tax_id, loyalty_points, created_at \
             FROM identity.customer_profiles WHERE id = $1",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        match row {
            Some(row) => Ok(Some(hydrate_customer_profile(&self.pool, row).await?)),
            None => Ok(None),
        }
    }

    async fn find_by_user(&self, user_id: UserId) -> Result<Option<CustomerProfile>, DomainError> {
        let row: Option<CustomerProfileRow> = sqlx::query_as(
            "SELECT id, user_id, company_name, tax_id, loyalty_points, created_at \
             FROM identity.customer_profiles WHERE user_id = $1",
        )
        .bind(user_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        match row {
            Some(row) => Ok(Some(hydrate_customer_profile(&self.pool, row).await?)),
            None => Ok(None),
        }
    }

    async fn update(&self, profile: &CustomerProfile) -> Result<(), DomainError> {
        let mut transaction = self.pool.begin().await.map_err(map_db_error)?;
        let affected = sqlx::query(
            "UPDATE identity.customer_profiles \
             SET company_name = $2, tax_id = $3, loyalty_points = $4 WHERE id = $1",
        )
        .bind(profile.id.0)
        .bind(&profile.company_name)
        .bind(&profile.tax_id)
        .bind(i64::try_from(profile.loyalty_points).map_err(|_| {
            DomainError::validation("customer loyalty points exceed PostgreSQL bigint range")
        })?)
        .execute(&mut *transaction)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if affected == 0 {
            return Err(map_row_absent());
        }
        sqlx::query("DELETE FROM identity.customer_addresses WHERE customer_profile_id = $1")
            .bind(profile.id.0)
            .execute(&mut *transaction)
            .await
            .map_err(map_db_error)?;
        insert_customer_addresses(&mut transaction, profile).await?;
        transaction.commit().await.map_err(map_db_error)
    }
}

#[derive(Clone)]
pub struct PgNotificationRepository {
    pool: PgPool,
}

impl PgNotificationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct NotificationRow {
    id: Uuid,
    recipient_id: Uuid,
    channel: String,
    title: String,
    body: String,
    status: String,
    created_at: DateTime<Utc>,
    sent_at: Option<DateTime<Utc>>,
}

impl NotificationRow {
    fn into_domain(self) -> Result<Notification, DomainError> {
        Ok(Notification {
            id: NotificationId(self.id),
            recipient_id: self.recipient_id,
            channel: self.channel.parse::<NotificationChannel>()?,
            title: self.title,
            body: self.body,
            status: self.status.parse::<NotificationStatus>()?,
            created_at: self.created_at,
            sent_at: self.sent_at,
        })
    }
}

#[async_trait]
impl NotificationRepository for PgNotificationRepository {
    async fn create(&self, notification: &Notification) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO notifications.notifications \
             (id, recipient_id, channel, title, body, status, created_at, sent_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
        )
        .bind(notification.id.0)
        .bind(notification.recipient_id)
        .bind(notification.channel.as_str())
        .bind(&notification.title)
        .bind(&notification.body)
        .bind(notification.status.as_str())
        .bind(notification.created_at)
        .bind(notification.sent_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: NotificationId) -> Result<Option<Notification>, DomainError> {
        let row: Option<NotificationRow> = sqlx::query_as(
            "SELECT id, recipient_id, channel, title, body, status, created_at, sent_at \
             FROM notifications.notifications WHERE id = $1",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        row.map(NotificationRow::into_domain).transpose()
    }

    async fn list_for_recipient(
        &self,
        recipient_id: Uuid,
    ) -> Result<Vec<Notification>, DomainError> {
        let rows: Vec<NotificationRow> = sqlx::query_as(
            "SELECT id, recipient_id, channel, title, body, status, created_at, sent_at \
             FROM notifications.notifications WHERE recipient_id = $1 ORDER BY created_at DESC",
        )
        .bind(recipient_id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;
        rows.into_iter().map(NotificationRow::into_domain).collect()
    }

    async fn update(&self, notification: &Notification) -> Result<(), DomainError> {
        let affected = sqlx::query(
            "UPDATE notifications.notifications SET status = $2, sent_at = $3 WHERE id = $1",
        )
        .bind(notification.id.0)
        .bind(notification.status.as_str())
        .bind(notification.sent_at)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if affected == 0 {
            return Err(map_row_absent());
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct PgTrackingRepository {
    pool: PgPool,
}

impl PgTrackingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct TrackingSessionRow {
    id: Uuid,
    courier_id: Uuid,
    status: String,
    started_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
}

impl TrackingSessionRow {
    fn into_domain(self) -> Result<TrackingSession, DomainError> {
        Ok(TrackingSession {
            id: self.id,
            courier_id: self.courier_id,
            status: self.status.parse::<TrackingSessionStatus>()?,
            started_at: self.started_at,
            ended_at: self.ended_at,
        })
    }
}

#[async_trait]
impl TrackingRepository for PgTrackingRepository {
    async fn record_point(&self, point: &TrackingPoint) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO tracking.location_points \
             (id, courier_id, latitude, longitude, speed_kmh, battery_pct, recorded_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(point.id)
        .bind(point.courier_id)
        .bind(point.location.latitude)
        .bind(point.location.longitude)
        .bind(point.speed_kmh)
        .bind(point.battery_pct.map(i16::from))
        .bind(point.recorded_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    async fn create_session(&self, session: &TrackingSession) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO tracking.sessions (id, courier_id, status, started_at, ended_at) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(session.id)
        .bind(session.courier_id)
        .bind(session.status.as_str())
        .bind(session.started_at)
        .bind(session.ended_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    async fn find_session(&self, id: Uuid) -> Result<Option<TrackingSession>, DomainError> {
        let row: Option<TrackingSessionRow> = sqlx::query_as(
            "SELECT id, courier_id, status, started_at, ended_at FROM tracking.sessions WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        row.map(TrackingSessionRow::into_domain).transpose()
    }

    async fn find_active_session_for_courier(
        &self,
        courier_id: Uuid,
    ) -> Result<Option<TrackingSession>, DomainError> {
        let row: Option<TrackingSessionRow> = sqlx::query_as(
            "SELECT id, courier_id, status, started_at, ended_at FROM tracking.sessions \
             WHERE courier_id = $1 AND status = 'active'",
        )
        .bind(courier_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        row.map(TrackingSessionRow::into_domain).transpose()
    }

    async fn update_session(&self, session: &TrackingSession) -> Result<(), DomainError> {
        let affected =
            sqlx::query("UPDATE tracking.sessions SET status = $2, ended_at = $3 WHERE id = $1")
                .bind(session.id)
                .bind(session.status.as_str())
                .bind(session.ended_at)
                .execute(&self.pool)
                .await
                .map_err(map_db_error)?
                .rows_affected();
        if affected == 0 {
            return Err(map_row_absent());
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct PgWebhookRepository {
    pool: PgPool,
}
impl PgWebhookRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
#[derive(FromRow)]
struct WebhookRow {
    id: Uuid,
    tenant_id: Uuid,
    endpoint_url: String,
    event_types: Vec<String>,
    secret_hash: String,
    enabled: bool,
    created_at: DateTime<Utc>,
}
impl From<WebhookRow> for WebhookSubscription {
    fn from(row: WebhookRow) -> Self {
        Self {
            id: row.id,
            tenant_id: TenantId(row.tenant_id),
            endpoint_url: row.endpoint_url,
            event_types: row.event_types,
            secret_hash: row.secret_hash,
            enabled: row.enabled,
            created_at: row.created_at,
        }
    }
}
#[async_trait]
impl WebhookRepository for PgWebhookRepository {
    async fn create(&self, item: &WebhookSubscription) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO integrations.webhooks (id, tenant_id, endpoint_url, event_types, secret_hash, enabled, created_at) VALUES ($1,$2,$3,$4,$5,$6,$7)")
            .bind(item.id).bind(item.tenant_id.0).bind(&item.endpoint_url).bind(&item.event_types).bind(&item.secret_hash).bind(item.enabled).bind(item.created_at).execute(&self.pool).await.map(|_| ()).map_err(map_db_error)
    }
    async fn list_for_tenant(
        &self,
        tenant_id: TenantId,
    ) -> Result<Vec<WebhookSubscription>, DomainError> {
        let rows: Vec<WebhookRow> = sqlx::query_as("SELECT id, tenant_id, endpoint_url, event_types, secret_hash, enabled, created_at FROM integrations.webhooks WHERE tenant_id=$1 ORDER BY created_at DESC").bind(tenant_id.0).fetch_all(&self.pool).await.map_err(map_db_error)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
    async fn delete(&self, tenant_id: TenantId, id: Uuid) -> Result<(), DomainError> {
        if sqlx::query("DELETE FROM integrations.webhooks WHERE id=$1 AND tenant_id=$2")
            .bind(id)
            .bind(tenant_id.0)
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?
            .rows_affected()
            == 0
        {
            return Err(map_row_absent());
        }
        Ok(())
    }
}

#[derive(FromRow)]
struct CourierRow {
    id: Uuid,
    name: String,
    vehicle: String,
    status: String,
    current_lat: Option<f64>,
    current_lon: Option<f64>,
    registered_at: DateTime<Utc>,
}

impl CourierRow {
    fn into_domain(self) -> Result<Courier, DomainError> {
        let current_location = match (self.current_lat, self.current_lon) {
            (Some(lat), Some(lon)) => Some(Location::new(lat, lon)?),
            _ => None,
        };
        Ok(Courier {
            id: self.id,
            name: self.name,
            vehicle: self.vehicle.parse()?,
            status: self.status.parse()?,
            current_location,
            registered_at: self.registered_at,
        })
    }
}

#[derive(Clone)]
pub struct PgCourierRepository {
    pool: PgPool,
}

impl PgCourierRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

const COURIER_COLUMNS: &str = "id, name, vehicle, status, current_lat, current_lon, registered_at";

#[async_trait]
impl CourierRepository for PgCourierRepository {
    async fn create(&self, courier: &Courier) -> Result<(), DomainError> {
        let mut query = sqlx::query(
            "INSERT INTO couriers.couriers (id, name, vehicle, status, current_lat, current_lon, \
             registered_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(courier.id)
        .bind(&courier.name)
        .bind(courier.vehicle.as_str())
        .bind(courier.status.as_str());
        query = match &courier.current_location {
            Some(location) => query
                .bind(Some(location.latitude))
                .bind(Some(location.longitude)),
            None => query.bind(None::<f64>).bind(None::<f64>),
        };
        query = query.bind(courier.registered_at);
        query
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Courier>, DomainError> {
        let row: Option<CourierRow> = sqlx::query_as(&format!(
            "SELECT {COURIER_COLUMNS} FROM couriers.couriers WHERE id = $1"
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        row.map(CourierRow::into_domain).transpose()
    }

    async fn list_all(&self) -> Result<Vec<Courier>, DomainError> {
        let rows: Vec<CourierRow> = sqlx::query_as(&format!(
            "SELECT {COURIER_COLUMNS} FROM couriers.couriers ORDER BY registered_at ASC"
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;
        rows.into_iter().map(CourierRow::into_domain).collect()
    }

    async fn list_available(&self) -> Result<Vec<Courier>, DomainError> {
        let rows: Vec<CourierRow> = sqlx::query_as(&format!(
            "SELECT {COURIER_COLUMNS} FROM couriers.couriers WHERE status = 'available' \
             ORDER BY registered_at ASC"
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(map_db_error)?;
        rows.into_iter().map(CourierRow::into_domain).collect()
    }

    async fn update(&self, courier: &Courier) -> Result<(), DomainError> {
        let mut query = sqlx::query(
            "UPDATE couriers.couriers SET status = $2, current_lat = $3, current_lon = $4 \
             WHERE id = $1",
        )
        .bind(courier.id)
        .bind(courier.status.as_str());
        query = match &courier.current_location {
            Some(location) => query
                .bind(Some(location.latitude))
                .bind(Some(location.longitude)),
            None => query.bind(None::<f64>).bind(None::<f64>),
        };
        let affected = query
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?
            .rows_affected();
        if affected == 0 {
            return Err(map_row_absent());
        }
        Ok(())
    }
}

#[derive(FromRow)]
struct AssignmentRow {
    id: Uuid,
    order_id: Uuid,
    courier_id: Uuid,
    status: String,
    assigned_at: DateTime<Utc>,
}

impl AssignmentRow {
    fn into_domain(self) -> Result<Assignment, DomainError> {
        Ok(Assignment {
            id: self.id,
            order_id: OrderId(self.order_id),
            courier_id: self.courier_id,
            status: self.status.parse()?,
            assigned_at: self.assigned_at,
        })
    }
}

#[derive(Clone)]
pub struct PgAssignmentRepository {
    pool: PgPool,
}

impl PgAssignmentRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssignmentRepository for PgAssignmentRepository {
    async fn create(&self, assignment: &Assignment) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO dispatch.assignments (id, order_id, courier_id, status, assigned_at) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(assignment.id)
        .bind(assignment.order_id.0)
        .bind(assignment.courier_id)
        .bind(assignment.status.as_str())
        .bind(assignment.assigned_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    async fn find_by_order(&self, order_id: OrderId) -> Result<Option<Assignment>, DomainError> {
        let row: Option<AssignmentRow> = sqlx::query_as(
            "SELECT id, order_id, courier_id, status, assigned_at \
             FROM dispatch.assignments WHERE order_id = $1",
        )
        .bind(order_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        row.map(AssignmentRow::into_domain).transpose()
    }

    async fn update(&self, assignment: &Assignment) -> Result<(), DomainError> {
        let affected = sqlx::query("UPDATE dispatch.assignments SET status = $2 WHERE id = $1")
            .bind(assignment.id)
            .bind(assignment.status.as_str())
            .execute(&self.pool)
            .await
            .map_err(map_db_error)?
            .rows_affected();
        if affected == 0 {
            return Err(map_row_absent());
        }
        Ok(())
    }
}

#[derive(FromRow)]
struct UserRow {
    id: Uuid,
    email: String,
    phone: Option<String>,
    display_name: String,
    role: String,
    status: String,
    created_at: DateTime<Utc>,
}

impl UserRow {
    fn into_domain(self) -> Result<User, DomainError> {
        Ok(User {
            id: UserId(self.id),
            email: self.email,
            phone: self.phone,
            display_name: self.display_name,
            role: self.role.parse()?,
            status: self.status.parse()?,
            created_at: self.created_at,
        })
    }
}

#[derive(Clone)]
pub struct PgUserRepository {
    pool: PgPool,
}

impl PgUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(&self, user: &User) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO identity.users (id, email, phone, display_name, role, status, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(user.id.0)
        .bind(&user.email)
        .bind(&user.phone)
        .bind(&user.display_name)
        .bind(user.role.as_str())
        .bind(user.status.as_str())
        .bind(user.created_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    async fn find_by_id(&self, id: UserId) -> Result<Option<User>, DomainError> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, email, phone, display_name, role, status, created_at FROM identity.users WHERE id = $1",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        row.map(UserRow::into_domain).transpose()
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, DomainError> {
        let row: Option<UserRow> = sqlx::query_as(
            "SELECT id, email, phone, display_name, role, status, created_at FROM identity.users WHERE LOWER(email) = LOWER($1)",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        row.map(UserRow::into_domain).transpose()
    }

    async fn update(&self, user: &User) -> Result<(), DomainError> {
        let affected = sqlx::query(
            "UPDATE identity.users SET phone = $2, display_name = $3, role = $4, status = $5 WHERE id = $1",
        )
        .bind(user.id.0)
        .bind(&user.phone)
        .bind(&user.display_name)
        .bind(user.role.as_str())
        .bind(user.status.as_str())
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if affected == 0 {
            return Err(map_row_absent());
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct PgTenantRepository {
    pool: PgPool,
}

impl PgTenantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct TenantRow {
    id: Uuid,
    name: String,
    created_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct TenantMembershipRow {
    tenant_id: Uuid,
    user_id: Uuid,
    role: String,
    joined_at: DateTime<Utc>,
}

#[async_trait]
impl TenantRepository for PgTenantRepository {
    async fn create_tenant(&self, tenant: &TenantCompany, slug: &str) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO tenancy.tenants (id, name, slug, status, created_at) VALUES ($1, $2, $3, 'active', $4)",
        )
        .bind(tenant.id.0)
        .bind(&tenant.company_name)
        .bind(slug)
        .bind(tenant.created_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<TenantCompany>, DomainError> {
        let row: Option<TenantRow> = sqlx::query_as(
            "SELECT id, name, created_at FROM tenancy.tenants WHERE slug = $1 AND status = 'active'",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(row.map(|row| TenantCompany {
            id: TenantId(row.id),
            company_name: row.name,
            category: "Logistics".into(),
            created_at: row.created_at,
        }))
    }

    async fn add_member(&self, membership: &TenantMembership) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO tenancy.tenant_members (tenant_id, user_id, role, joined_at) VALUES ($1, $2, $3, $4)",
        )
        .bind(membership.tenant_id.0)
        .bind(membership.user_id.0)
        .bind(membership.role.as_str())
        .bind(membership.joined_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    async fn find_membership(
        &self,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<Option<TenantMembership>, DomainError> {
        let row: Option<TenantMembershipRow> = sqlx::query_as(
            "SELECT tenant_id, user_id, role, joined_at FROM tenancy.tenant_members WHERE tenant_id = $1 AND user_id = $2",
        )
        .bind(tenant_id.0)
        .bind(user_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        row.map(|row| {
            Ok(TenantMembership {
                tenant_id: TenantId(row.tenant_id),
                user_id: UserId(row.user_id),
                role: row.role.parse::<TenantMemberRole>()?,
                joined_at: row.joined_at,
            })
        })
        .transpose()
    }

    async fn bind_courier(&self, tenant_id: TenantId, courier_id: Uuid) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO tenancy.courier_tenants (courier_id, tenant_id) VALUES ($1, $2)")
            .bind(courier_id)
            .bind(tenant_id.0)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(map_db_error)
    }

    async fn bind_order(&self, tenant_id: TenantId, order_id: OrderId) -> Result<(), DomainError> {
        sqlx::query("INSERT INTO tenancy.order_tenants (order_id, tenant_id) VALUES ($1, $2)")
            .bind(order_id.0)
            .bind(tenant_id.0)
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(map_db_error)
    }

    async fn find_courier_tenant(&self, courier_id: Uuid) -> Result<Option<TenantId>, DomainError> {
        let tenant_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT tenant_id FROM tenancy.courier_tenants WHERE courier_id = $1",
        )
        .bind(courier_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(tenant_id.map(TenantId))
    }

    async fn find_order_tenant(&self, order_id: OrderId) -> Result<Option<TenantId>, DomainError> {
        let tenant_id: Option<Uuid> =
            sqlx::query_scalar("SELECT tenant_id FROM tenancy.order_tenants WHERE order_id = $1")
                .bind(order_id.0)
                .fetch_optional(&self.pool)
                .await
                .map_err(map_db_error)?;
        Ok(tenant_id.map(TenantId))
    }
}

#[derive(Clone)]
pub struct PgCredentialRepository {
    pool: PgPool,
}

impl PgCredentialRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[derive(FromRow)]
struct CredentialRow {
    user_id: Uuid,
    password_hash: String,
    password_changed_at: DateTime<Utc>,
}

#[derive(FromRow)]
struct RefreshSessionRow {
    id: Uuid,
    user_id: Uuid,
    tenant_id: Uuid,
    token_hash: String,
    expires_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
}

#[async_trait]
impl CredentialRepository for PgCredentialRepository {
    async fn save_credential(&self, credential: &Credential) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO identity.credentials (user_id, password_hash, password_changed_at) \
             VALUES ($1, $2, $3) \
             ON CONFLICT (user_id) DO UPDATE SET password_hash = EXCLUDED.password_hash, \
             password_changed_at = EXCLUDED.password_changed_at",
        )
        .bind(credential.user_id.0)
        .bind(&credential.password_hash)
        .bind(credential.password_changed_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    async fn find_credential(&self, user_id: UserId) -> Result<Option<Credential>, DomainError> {
        let row: Option<CredentialRow> = sqlx::query_as(
            "SELECT user_id, password_hash, password_changed_at FROM identity.credentials WHERE user_id = $1",
        )
        .bind(user_id.0)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(row.map(|row| Credential {
            user_id: UserId(row.user_id),
            password_hash: row.password_hash,
            password_changed_at: row.password_changed_at,
        }))
    }

    async fn save_refresh_session(&self, session: &RefreshSession) -> Result<(), DomainError> {
        sqlx::query(
            "INSERT INTO identity.refresh_sessions (id, user_id, tenant_id, token_hash, expires_at, revoked_at, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(session.id)
        .bind(session.user_id.0)
        .bind(session.tenant_id.0)
        .bind(&session.token_hash)
        .bind(session.expires_at)
        .bind(session.revoked_at)
        .bind(session.created_at)
        .execute(&self.pool)
        .await
        .map(|_| ())
        .map_err(map_db_error)
    }

    async fn find_refresh_session(
        &self,
        token_hash: &str,
    ) -> Result<Option<RefreshSession>, DomainError> {
        let row: Option<RefreshSessionRow> = sqlx::query_as(
            "SELECT id, user_id, tenant_id, token_hash, expires_at, revoked_at, created_at \
             FROM identity.refresh_sessions WHERE token_hash = $1",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(row.map(|row| RefreshSession {
            id: row.id,
            user_id: UserId(row.user_id),
            tenant_id: TenantId(row.tenant_id),
            token_hash: row.token_hash,
            expires_at: row.expires_at,
            revoked_at: row.revoked_at,
            created_at: row.created_at,
        }))
    }

    async fn revoke_refresh_session(&self, id: Uuid) -> Result<(), DomainError> {
        let affected = sqlx::query(
            "UPDATE identity.refresh_sessions SET revoked_at = COALESCE(revoked_at, now()) WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?
        .rows_affected();
        if affected == 0 {
            return Err(map_row_absent());
        }
        Ok(())
    }
}

// Re-exported for the migration runner and tests.
pub type PgPoolHandle = PgPool;
pub use sqlx::postgres::PgPoolOptions;

#[cfg(test)]
mod tests {
    use qervon_domain::{AssignmentStatus, CourierStatus, OrderStatus, VehicleType};

    #[test]
    fn enum_round_trip_strings_match_migrations() {
        assert_eq!(OrderStatus::Pending.as_str(), "pending");
        assert_eq!(
            "courier_assigned".parse::<OrderStatus>().unwrap(),
            OrderStatus::CourierAssigned
        );
        assert_eq!(CourierStatus::Available.as_str(), "available");
        assert_eq!(VehicleType::Motorcycle.as_str(), "motorcycle");
        assert_eq!(AssignmentStatus::Assigned.as_str(), "assigned");
    }
}
