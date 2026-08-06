// =============================================================================
// File:           backend/crates/application/tests/dispatch_flow.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   End-to-end dispatch flow against in-memory adapters.
//
// Specification:
//   QAS-000002, QAS-000003, QLS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use qervon_application::{
    CourierService, CreateOrderInput, DispatchService, OrderService, RegisterCourierInput,
};
use qervon_domain::{Address, CourierStatus, Location, Money, OrderStatus, VehicleType};
use qervon_infrastructure::InMemoryStore;
use uuid::Uuid;

fn order_input() -> CreateOrderInput {
    CreateOrderInput {
        customer_id: Uuid::now_v7(),
        pickup: Address {
            location: Location::new(41.0, 29.0).unwrap(),
            label: Some("pickup".into()),
        },
        dropoff: Address {
            location: Location::new(41.1, 29.1).unwrap(),
            label: Some("dropoff".into()),
        },
        fare: Money::new(1_500, "TRY").unwrap(),
    }
}

#[tokio::test]
async fn full_dispatch_flow_returns_courier_to_available() {
    let store = InMemoryStore::new();
    let orders = OrderService::new(store.order_repository());
    let couriers = CourierService::new(store.courier_repository());
    let dispatch = DispatchService::new(
        store.order_repository(),
        store.courier_repository(),
        store.assignment_repository(),
    );

    let courier = couriers
        .register(RegisterCourierInput {
            id: Uuid::now_v7(),
            name: "Ali Kurye".to_string(),
            vehicle: VehicleType::Car,
        })
        .await
        .expect("register courier");
    let mut courier = couriers
        .update_location(courier.id, Location::new(41.0, 29.0).unwrap())
        .await
        .expect("update location");

    let order = orders.create(order_input()).await.expect("create order");
    assert_eq!(order.status, OrderStatus::Pending);

    let assignment = dispatch.assign(order.id, courier.id).await.expect("assign");
    assert_eq!(assignment.order_id, order.id);

    let assigned = orders.get(order.id).await.expect("fetch assigned");
    assert_eq!(assigned.status, OrderStatus::CourierAssigned);
    courier = couriers.get(courier.id).await.expect("fetch courier");
    assert_eq!(courier.status, CourierStatus::Busy);

    let transit = dispatch
        .start_transit(order.id)
        .await
        .expect("start transit");
    assert_eq!(transit.status, OrderStatus::InTransit);

    let delivered = dispatch.deliver(order.id).await.expect("deliver");
    assert_eq!(delivered.status, OrderStatus::Delivered);
    assert!(delivered.delivered_at.is_some());

    let freed = couriers.get(courier.id).await.expect("fetch freed courier");
    assert_eq!(freed.status, CourierStatus::Available);
}

#[tokio::test]
async fn auto_assign_selects_closest_available_courier() {
    let store = InMemoryStore::new();
    let couriers = CourierService::new(store.courier_repository());
    let orders = OrderService::new(store.order_repository());
    let dispatch = DispatchService::new(
        store.order_repository(),
        store.courier_repository(),
        store.assignment_repository(),
    );

    let far = couriers
        .register(RegisterCourierInput {
            id: Uuid::now_v7(),
            name: "Far".to_string(),
            vehicle: VehicleType::Bicycle,
        })
        .await
        .expect("register far courier");
    couriers
        .update_location(far.id, Location::new(39.9, 32.8).unwrap())
        .await
        .expect("far location");

    let near = couriers
        .register(RegisterCourierInput {
            id: Uuid::now_v7(),
            name: "Near".to_string(),
            vehicle: VehicleType::Motorcycle,
        })
        .await
        .expect("register near courier");
    couriers
        .update_location(near.id, Location::new(41.0, 29.0).unwrap())
        .await
        .expect("near location");

    let order = orders.create(order_input()).await.expect("create order");
    let assignment = dispatch.auto_assign(order.id).await.expect("auto assign");
    assert_eq!(assignment.courier_id, near.id);
}

#[tokio::test]
async fn delivering_unknown_order_fails() {
    let store = InMemoryStore::new();
    let dispatch = DispatchService::new(
        store.order_repository(),
        store.courier_repository(),
        store.assignment_repository(),
    );

    let result = dispatch.deliver(qervon_domain::OrderId::new()).await;
    assert!(result.is_err());
}
