// =============================================================================
// File:           backend/crates/domain/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Qervon domain kernel: entities, aggregates, value objects, and ports.
//   This crate has no infrastructure dependencies; business truth lives here.
//
// Specification:
//   QAS-000001 through QAS-000006, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

pub mod billing;
pub mod courier;
pub mod courier_wallet;
pub mod customer;
pub mod customer_feedback;
pub mod dispatch;
pub mod error;
pub mod fleet;
pub mod location;
pub mod money;
pub mod notification;
pub mod order;
pub mod repository;
pub mod route_history;
pub mod tenant;
pub mod tracking;
pub mod user;
pub mod warehouse_hub;

pub use billing::{CourierPayout, Invoice, InvoiceId, InvoiceStatus, PayoutStatus};
pub use courier::{Courier, CourierStatus, VehicleType};
pub use courier_wallet::{CourierWallet, WalletTransaction, WalletTransactionType};
pub use customer_feedback::{CustomerRating, SupportTicket, TicketStatus};
pub use route_history::{CourierPlaybackTrack, RouteBreadcrumb};
pub use tenant::{TenantCompany, TenantId, BranchId, TenantBranch};
pub use warehouse_hub::{WarehouseHub, HubManifestAssignment};
pub use customer::{CustomerId, CustomerProfile, SavedAddress};
pub use dispatch::{Assignment, AssignmentStatus};
pub use error::DomainError;
pub use fleet::{Vehicle, VehicleId, VehicleStatus};
pub use location::Location;
pub use money::Money;
pub use notification::{Notification, NotificationChannel, NotificationId, NotificationStatus};
pub use order::{Address, Order, OrderId, OrderStatus};
pub use repository::{
    AssignmentRepository, CourierPayoutRepository, CourierRepository, CustomerRepository,
    InvoiceRepository, NotificationRepository, OrderRepository, TrackingRepository,
    UserRepository, VehicleRepository,
};
pub use tracking::{TrackingPoint, TrackingSession, TrackingSessionStatus};
pub use user::{User, UserId, UserRole, UserStatus};

