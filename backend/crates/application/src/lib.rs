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

pub mod courier_service;
pub mod dispatch_service;
pub mod error;
pub mod order_service;

pub use courier_service::{CourierService, RegisterCourierInput};
pub use dispatch_service::DispatchService;
pub use error::ApplicationError;
pub use order_service::{CreateOrderInput, OrderService};
