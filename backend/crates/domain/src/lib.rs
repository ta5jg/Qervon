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

pub mod courier;
pub mod dispatch;
pub mod error;
pub mod location;
pub mod money;
pub mod order;
pub mod repository;

pub use courier::{Courier, CourierStatus, VehicleType};
pub use dispatch::{Assignment, AssignmentStatus};
pub use error::DomainError;
pub use location::Location;
pub use money::Money;
pub use order::{Address, Order, OrderId, OrderStatus};
pub use repository::{AssignmentRepository, CourierRepository, OrderRepository};
