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
use chrono::{DateTime, Utc};
use qervon_domain::{
    Address, Assignment, AssignmentRepository, Courier, CourierRepository, DomainError, Location,
    Money, Order, OrderId, OrderRepository,
};
use sqlx::{FromRow, PgPool};
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
        query.execute(&self.pool).await.map(|_| ()).map_err(map_db_error)
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
