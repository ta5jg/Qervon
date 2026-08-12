// =============================================================================
// File:           backend/crates/api-contracts/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Qervon API wire contracts: request and response DTOs for the vertical slice.
//
// Specification:
//   QAS-000005, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, NaiveDate, Utc};
use qervon_domain::{
    Address, Assignment, AssignmentStatus, Coupon, Courier, CourierStatus, CourierWallet,
    CustomerRating, DevicePushToken, Location, Money, Order, OrderStatus, PushPlatform,
    SupportTicket, TicketStatus, Vehicle, VehicleStatus, VehicleType, WalletTransaction,
    WalletTransactionType, OFFER_TTL,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------- Requests ----------

#[derive(Debug, Clone, Deserialize)]
pub struct CreateOrderRequest {
    pub customer_id: Uuid,
    pub pickup: AddressDto,
    pub dropoff: AddressDto,
    pub fare_amount_minor: i64,
    pub fare_currency: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCustomerOrderRequest {
    pub pickup: AddressDto,
    pub dropoff: AddressDto,
    /// Optional promo coupon code to apply to the fare before order
    /// creation. Redemption is recorded immediately once validated.
    pub coupon_code: Option<String>,
    /// One of "cash", "card", "qr", "wallet". Card/QR/wallet only record
    /// the chosen method; no real payment gateway is integrated yet.
    pub payment_method: Option<String>,
    /// Free-form delivery instructions (e.g. "kapıcıya bırakın").
    pub delivery_note: Option<String>,
    /// A contact number for the courier to reach at the dropoff.
    pub contact_phone: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssignCourierRequest {
    /// When omitted, the closest available courier is selected automatically.
    pub courier_id: Option<Uuid>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterCourierRequest {
    pub id: Option<Uuid>,
    pub name: String,
    pub vehicle: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateLocationRequest {
    pub latitude: f64,
    pub longitude: f64,
    pub speed_kmh: Option<f64>,
    pub battery_pct: Option<u8>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SetCourierAvailabilityRequest {
    pub online: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterUserRequest {
    pub email: String,
    pub display_name: String,
    pub role: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RateOrderRequest {
    pub rating_stars: u8,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpenSupportTicketRequest {
    pub order_id: Option<Uuid>,
    pub subject: String,
    pub message: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterPushDeviceRequest {
    /// "ios" or "android".
    pub platform: String,
    pub device_token: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct DevicePushTokenResponse {
    pub id: Uuid,
    pub platform: PushPlatform,
    pub device_token: String,
    pub created_at: DateTime<Utc>,
}

impl From<&DevicePushToken> for DevicePushTokenResponse {
    fn from(token: &DevicePushToken) -> Self {
        Self {
            id: token.id,
            platform: token.platform,
            device_token: token.device_token.clone(),
            created_at: token.created_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateCouponRequest {
    pub code: String,
    pub discount_percent: f64,
    pub max_discount_minor: i64,
    pub valid_until: DateTime<Utc>,
    pub usage_limit: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CouponResponse {
    pub id: Uuid,
    pub code: String,
    pub discount_percent: f64,
    pub max_discount_minor: i64,
    pub valid_until: DateTime<Utc>,
    pub usage_limit: u32,
    pub used_count: u32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<&Coupon> for CouponResponse {
    fn from(coupon: &Coupon) -> Self {
        Self {
            id: coupon.id,
            code: coupon.code.clone(),
            discount_percent: coupon.discount_percent,
            max_discount_minor: coupon.max_discount_minor,
            valid_until: coupon.valid_until,
            usage_limit: coupon.usage_limit,
            used_count: coupon.used_count,
            is_active: coupon.is_active,
            created_at: coupon.created_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegisterVehicleRequest {
    pub plate_number: String,
    pub vehicle_type: String,
    /// ISO 8601 calendar date (`YYYY-MM-DD`), when known.
    pub insurance_expiry: Option<NaiveDate>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AssignVehicleRequest {
    pub courier_id: Uuid,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdatePricingRequest {
    pub base_fare_minor: i64,
    pub per_km_rate_minor: i64,
    pub minimum_fare_minor: i64,
    pub currency: String,
}

// ---------- Value-object DTOs ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressDto {
    pub latitude: f64,
    pub longitude: f64,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MoneyDto {
    pub amount_minor: i64,
    pub currency: String,
}

// ---------- Responses ----------

#[derive(Debug, Clone, Serialize)]
pub struct OrderResponse {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub pickup: AddressDto,
    pub dropoff: AddressDto,
    pub status: OrderStatus,
    pub fare: MoneyDto,
    pub assigned_courier_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub returned_at: Option<DateTime<Utc>>,
    pub payment_method: Option<qervon_domain::PaymentMethod>,
    pub payment_collected: bool,
    pub delivery_note: Option<String>,
    pub contact_phone: Option<String>,
}

/// A non-binding fare estimate for a pickup/dropoff pair. The order
/// creation endpoint always recomputes the authoritative fare itself, so a
/// client can never manipulate the final charge by round-tripping this
/// value.
#[derive(Debug, Clone, Serialize)]
pub struct FareQuoteResponse {
    pub fare_amount_minor: i64,
    pub currency: String,
    pub distance_km: f64,
}

/// Estimated minutes until the assigned courier reaches the relevant leg
/// (pickup while `courier_assigned`, dropoff while `in_transit`). Uses the
/// same distance/vehicle-type estimate as the AI Dispatcher — no real
/// traffic data is factored in.
#[derive(Debug, Clone, Serialize)]
pub struct EtaResponse {
    pub eta_minutes: f64,
    pub distance_km: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PricingResponse {
    pub base_fare_minor: i64,
    pub per_km_rate_minor: i64,
    pub minimum_fare_minor: i64,
    pub currency: String,
    pub updated_at: DateTime<Utc>,
}

impl From<&qervon_domain::DeliveryPricing> for PricingResponse {
    fn from(pricing: &qervon_domain::DeliveryPricing) -> Self {
        Self {
            base_fare_minor: pricing.base_fare_minor,
            per_km_rate_minor: pricing.per_km_rate_minor,
            minimum_fare_minor: pricing.minimum_fare_minor,
            currency: pricing.currency.clone(),
            updated_at: pricing.updated_at,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CourierResponse {
    pub id: Uuid,
    pub name: String,
    pub vehicle: VehicleType,
    pub status: CourierStatus,
    pub current_location: Option<Location>,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct VehicleResponse {
    pub id: Uuid,
    pub plate_number: String,
    pub vehicle_type: VehicleType,
    pub status: VehicleStatus,
    pub assigned_courier_id: Option<Uuid>,
    pub insurance_expiry: Option<NaiveDate>,
    pub registered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CustomerRatingResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub customer_id: Uuid,
    pub courier_id: Uuid,
    pub rating_stars: u8,
    pub comment: Option<String>,
    pub photo_url: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SupportTicketResponse {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub order_id: Option<Uuid>,
    pub subject: String,
    pub message: String,
    pub status: TicketStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WalletTransactionResponse {
    pub id: Uuid,
    pub transaction_type: WalletTransactionType,
    pub amount_minor: i64,
    pub currency: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CourierWalletResponse {
    pub courier_id: Uuid,
    pub balance_minor: i64,
    pub total_earned_minor: i64,
    pub total_bonus_minor: i64,
    pub total_penalties_minor: i64,
    pub currency: String,
    pub transactions: Vec<WalletTransactionResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AssignmentResponse {
    pub id: Uuid,
    pub order_id: Uuid,
    pub courier_id: Uuid,
    pub status: AssignmentStatus,
    pub assigned_at: DateTime<Utc>,
    pub offered_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
}

/// A job offered to a courier who has not yet accepted or rejected it.
#[derive(Debug, Clone, Serialize)]
pub struct PendingOfferResponse {
    pub assignment_id: Uuid,
    pub order: OrderResponse,
    pub offered_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl PendingOfferResponse {
    pub fn new(assignment: &Assignment, order: &Order) -> Self {
        Self {
            assignment_id: assignment.id,
            order: order.into(),
            offered_at: assignment.offered_at,
            expires_at: assignment.offered_at + OFFER_TTL,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct OperationsOverviewResponse {
    pub active_orders: usize,
    pub pending_orders: usize,
    pub in_transit_orders: usize,
    pub returned_orders: usize,
    pub available_couriers: usize,
    pub busy_couriers: usize,
    pub offline_couriers: usize,
    pub delivered_revenue_by_currency: Vec<MoneyDto>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub display_name: String,
    pub role: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CustomerProfileResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub company_name: Option<String>,
    pub loyalty_points: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResponse {
    pub status: u16,
    pub title: String,
    pub detail: String,
}

// ---------- Conversions ----------

impl From<&Address> for AddressDto {
    fn from(address: &Address) -> Self {
        Self {
            latitude: address.location.latitude,
            longitude: address.location.longitude,
            label: address.label.clone(),
        }
    }
}

impl From<&Money> for MoneyDto {
    fn from(money: &Money) -> Self {
        Self {
            amount_minor: money.amount_minor,
            currency: money.currency.clone(),
        }
    }
}

impl From<&Order> for OrderResponse {
    fn from(order: &Order) -> Self {
        Self {
            id: order.id.0,
            customer_id: order.customer_id,
            pickup: (&order.pickup).into(),
            dropoff: (&order.dropoff).into(),
            status: order.status,
            fare: (&order.fare).into(),
            assigned_courier_id: order.assigned_courier_id,
            created_at: order.created_at,
            delivered_at: order.delivered_at,
            returned_at: order.returned_at,
            payment_method: order.payment_method,
            payment_collected: order.payment_collected,
            delivery_note: order.delivery_note.clone(),
            contact_phone: order.contact_phone.clone(),
        }
    }
}

impl From<&Courier> for CourierResponse {
    fn from(courier: &Courier) -> Self {
        Self {
            id: courier.id,
            name: courier.name.clone(),
            vehicle: courier.vehicle,
            status: courier.status,
            current_location: courier.current_location,
            registered_at: courier.registered_at,
        }
    }
}

impl From<&Vehicle> for VehicleResponse {
    fn from(vehicle: &Vehicle) -> Self {
        Self {
            id: vehicle.id.0,
            plate_number: vehicle.plate_number.clone(),
            vehicle_type: vehicle.vehicle_type,
            status: vehicle.status,
            assigned_courier_id: vehicle.assigned_courier_id,
            insurance_expiry: vehicle.insurance_expiry,
            registered_at: vehicle.registered_at,
        }
    }
}

impl From<&CustomerRating> for CustomerRatingResponse {
    fn from(rating: &CustomerRating) -> Self {
        Self {
            id: rating.id,
            order_id: rating.order_id,
            customer_id: rating.customer_id,
            courier_id: rating.courier_id,
            rating_stars: rating.rating_stars,
            comment: rating.comment.clone(),
            photo_url: rating.photo_url.clone(),
            created_at: rating.created_at,
        }
    }
}

impl From<&SupportTicket> for SupportTicketResponse {
    fn from(ticket: &SupportTicket) -> Self {
        Self {
            id: ticket.id,
            customer_id: ticket.customer_id,
            order_id: ticket.order_id,
            subject: ticket.subject.clone(),
            message: ticket.message.clone(),
            status: ticket.status,
            created_at: ticket.created_at,
        }
    }
}

impl From<&WalletTransaction> for WalletTransactionResponse {
    fn from(transaction: &WalletTransaction) -> Self {
        Self {
            id: transaction.id,
            transaction_type: transaction.transaction_type,
            amount_minor: transaction.amount_minor,
            currency: transaction.currency.clone(),
            description: transaction.description.clone(),
            created_at: transaction.created_at,
        }
    }
}

impl From<&CourierWallet> for CourierWalletResponse {
    fn from(wallet: &CourierWallet) -> Self {
        Self {
            courier_id: wallet.courier_id,
            balance_minor: wallet.balance_minor,
            total_earned_minor: wallet.total_earned_minor,
            total_bonus_minor: wallet.total_bonus_minor,
            total_penalties_minor: wallet.total_penalties_minor,
            currency: wallet.currency.clone(),
            transactions: wallet.transactions.iter().map(Into::into).collect(),
        }
    }
}

impl From<&Assignment> for AssignmentResponse {
    fn from(assignment: &Assignment) -> Self {
        Self {
            id: assignment.id,
            order_id: assignment.order_id.0,
            courier_id: assignment.courier_id,
            status: assignment.status,
            assigned_at: assignment.assigned_at,
            offered_at: assignment.offered_at,
            responded_at: assignment.responded_at,
        }
    }
}

impl From<&qervon_domain::User> for UserResponse {
    fn from(user: &qervon_domain::User) -> Self {
        Self {
            id: user.id.0,
            email: user.email.clone(),
            display_name: user.display_name.clone(),
            role: user.role.as_str().to_string(),
            status: user.status.as_str().to_string(),
            created_at: user.created_at,
        }
    }
}

impl From<&qervon_domain::CustomerProfile> for CustomerProfileResponse {
    fn from(profile: &qervon_domain::CustomerProfile) -> Self {
        Self {
            id: profile.id.0,
            user_id: profile.user_id.0,
            company_name: profile.company_name.clone(),
            loyalty_points: profile.loyalty_points,
            created_at: profile.created_at,
        }
    }
}
