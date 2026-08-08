// =============================================================================
// File:           backend/crates/application/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Qervon application kernel: use cases orchestrating domain aggregates.
//   Depends only on domain ports; adapters are injected by composition roots.
//
// Specification:
//   QAS-000001 through QAS-000006, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

pub mod ai_dispatcher;
pub mod billing_service;
pub mod bulk_order;
pub mod courier_leaderboard;
pub mod courier_service;
pub mod currency_exchange;
pub mod customer_service;
pub mod dispatch_service;
pub mod error;
pub mod field_service;
pub mod fleet_service;
pub mod notification_hub;
pub mod notification_service;
pub mod order_service;
pub mod parcel_sizing;
pub mod promo_coupon;
pub mod tax_invoicing;
pub mod tracking_service;
pub mod user_service;

pub use ai_dispatcher::{AiDispatcher, DispatchScore, WeatherCondition, TrafficContext};
pub use billing_service::{BillingService, CreateInvoiceInput, CreatePayoutInput};
pub use bulk_order::{BulkOrderParser, BulkOrderRow, WebhookPayload};
pub use courier_leaderboard::{CourierLeaderboardEngine, CourierLeaderboardEntry};
pub use currency_exchange::CurrencyExchangeEngine;
pub use field_service::{FieldServiceScheduler, FieldServiceAppointment, TimeSlotWindow};
pub use parcel_sizing::{ParcelSizingEngine, ParcelDimensions};
pub use promo_coupon::{PromoCouponEngine, PromoCoupon};
pub use tax_invoicing::{TaxInvoicingEngine, ElectronicInvoiceDraft};
pub use notification_hub::{NotificationHubManager, NotificationMessage, ChannelType};
pub use courier_service::{CourierService, RegisterCourierInput};
pub use customer_service::CustomerService;
pub use dispatch_service::DispatchService;
pub use error::ApplicationError;
pub use fleet_service::{FleetService, RegisterVehicleInput};
pub use notification_service::{NotificationService, SendNotificationInput};
pub use order_service::{CreateOrderInput, OrderService};
pub use tracking_service::TrackingService;
pub use user_service::{CreateUserInput, UserService};

