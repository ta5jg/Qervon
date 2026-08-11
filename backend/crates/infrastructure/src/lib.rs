// =============================================================================
// File:           backend/crates/infrastructure/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Qervon infrastructure kernel: concrete adapters for domain repository ports.
//   Provides in-memory adapters for tests/dev and Postgres adapters for runtime.
//
// Specification:
//   QAS-000001 through QAS-000006, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

pub mod memory;
pub mod postgres;

pub use memory::{
    InMemoryAssignmentRepository, InMemoryCourierRepository, InMemoryCredentialRepository,
    InMemoryCustomerRepository, InMemoryInvoiceRepository, InMemoryNotificationRepository,
    InMemoryOrderRepository, InMemoryPayoutRepository, InMemoryProofOfDeliveryRepository,
    InMemoryStore, InMemoryTenantRepository, InMemoryTrackingRepository, InMemoryUserRepository,
    InMemoryVehicleRepository, InMemoryWebhookRepository,
};
pub use postgres::{
    PgAssignmentRepository, PgCourierPayoutRepository, PgCourierRepository, PgCredentialRepository,
    PgCustomerRepository, PgInvoiceRepository, PgNotificationRepository, PgOrderRepository,
    PgProofOfDeliveryRepository, PgTenantRepository, PgTrackingRepository, PgUserRepository,
    PgVehicleRepository, PgWebhookRepository,
};
