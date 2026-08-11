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
pub mod cold_chain;
pub mod courier;
pub mod courier_shift;
pub mod courier_wallet;
pub mod credential;
pub mod customer;
pub mod customer_feedback;
pub mod dispatch;
pub mod error;
pub mod fleet;
pub mod location;
pub mod money;
pub mod notification;
pub mod order;
pub mod proof_of_delivery;
pub mod repository;
pub mod route_history;
pub mod tenant;
pub mod tracking;
pub mod user;
pub mod warehouse_hub;
pub mod webhook;

pub use billing::{CourierPayout, Invoice, InvoiceId, InvoiceStatus, PayoutStatus};
pub use cold_chain::ColdChainTelemetry;
pub use courier::{Courier, CourierStatus, VehicleType};
pub use courier_shift::{CourierShiftAssignment, ShiftType};
pub use courier_wallet::{CourierWallet, WalletTransaction, WalletTransactionType};
pub use credential::{Credential, RefreshSession};
pub use customer::{CustomerId, CustomerProfile, SavedAddress};
pub use customer_feedback::{CustomerRating, SupportTicket, TicketStatus};
pub use dispatch::{Assignment, AssignmentStatus};
pub use error::DomainError;
pub use fleet::{Vehicle, VehicleId, VehicleStatus};
pub use location::Location;
pub use money::Money;
pub use notification::{Notification, NotificationChannel, NotificationId, NotificationStatus};
pub use order::{Address, Order, OrderId, OrderStatus};
pub use proof_of_delivery::ProofOfDeliveryRecord;
pub use repository::{
    AssignmentRepository, CourierPayoutRepository, CourierRepository, CredentialRepository,
    CustomerRepository, InvoiceRepository, NotificationRepository, OrderRepository,
    ProofOfDeliveryRepository, TenantRepository, TrackingRepository, UserRepository,
    VehicleRepository, WebhookRepository,
};
pub use route_history::{CourierPlaybackTrack, RouteBreadcrumb};
pub use tenant::{
    BranchId, TenantBranch, TenantCompany, TenantId, TenantMemberRole, TenantMembership,
};
pub use tracking::{TrackingPoint, TrackingSession, TrackingSessionStatus};
pub use user::{User, UserId, UserRole, UserStatus};
pub use warehouse_hub::{HubManifestAssignment, WarehouseHub};
pub use webhook::WebhookSubscription;
