// =============================================================================
// File:           backend/crates/test-support/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Shared test fixtures for Qervon backend integration tests.
//
// Specification:
//   QAS-000001 through QAS-000006, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::Utc;
use qervon_domain::{Address, Courier, Location, Money, Order, OrderId, VehicleType};
use uuid::Uuid;

pub fn sample_address(label: &str) -> Address {
    Address {
        location: Location::new(41.0, 29.0).expect("valid location"),
        label: Some(label.to_string()),
    }
}

pub fn sample_order() -> Order {
    Order::create(
        OrderId::new(),
        Uuid::now_v7(),
        sample_address("pickup"),
        sample_address("dropoff"),
        Money::new(1_500, "TRY").expect("valid money"),
        Utc::now(),
        None,
        None,
    )
    .expect("valid order")
}

pub fn sample_courier(name: &str) -> Courier {
    Courier::create(Uuid::now_v7(), name, VehicleType::Motorcycle, Utc::now())
        .expect("valid courier")
}
