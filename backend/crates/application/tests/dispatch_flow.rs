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
        payment_method: None,
        delivery_note: None,
        contact_phone: None,
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
async fn offer_accepted_assigns_the_order_and_busies_the_courier() {
    let store = InMemoryStore::new();
    let couriers = CourierService::new(store.courier_repository());
    let orders = OrderService::new(store.order_repository());
    let dispatch = DispatchService::new(
        store.order_repository(),
        store.courier_repository(),
        store.assignment_repository(),
    );

    let courier = couriers
        .register(RegisterCourierInput {
            id: Uuid::now_v7(),
            name: "Teklif Kurye".to_string(),
            vehicle: VehicleType::Motorcycle,
        })
        .await
        .expect("register courier");
    couriers
        .update_location(courier.id, Location::new(41.0, 29.0).unwrap())
        .await
        .expect("courier location");

    let order = orders.create(order_input()).await.expect("create order");
    let offer = dispatch
        .offer_for_order(order.id)
        .await
        .expect("offer")
        .expect("a candidate exists");
    assert_eq!(offer.courier_id, courier.id);

    // While merely offered, order and courier are untouched.
    let still_pending = orders.get(order.id).await.expect("fetch order");
    assert_eq!(still_pending.status, OrderStatus::Pending);
    let still_available = couriers.get(courier.id).await.expect("fetch courier");
    assert_eq!(still_available.status, CourierStatus::Available);

    let (pending_offer, pending_order) = dispatch
        .find_pending_offer(courier.id)
        .await
        .expect("find pending offer")
        .expect("offer exists");
    assert_eq!(pending_offer.order_id, order.id);
    assert_eq!(pending_order.id, order.id);

    let accepted = dispatch
        .accept_offer(order.id, courier.id)
        .await
        .expect("accept offer");
    assert_eq!(accepted.status, OrderStatus::CourierAssigned);
    let busy = couriers.get(courier.id).await.expect("fetch courier");
    assert_eq!(busy.status, CourierStatus::Busy);

    // Once accepted, there is no longer a pending offer for this courier.
    let no_more_offer = dispatch
        .find_pending_offer(courier.id)
        .await
        .expect("find pending offer");
    assert!(no_more_offer.is_none());
}

#[tokio::test]
async fn rejecting_an_offer_leaves_the_order_pending_and_courier_available() {
    let store = InMemoryStore::new();
    let couriers = CourierService::new(store.courier_repository());
    let orders = OrderService::new(store.order_repository());
    let dispatch = DispatchService::new(
        store.order_repository(),
        store.courier_repository(),
        store.assignment_repository(),
    );

    let courier = couriers
        .register(RegisterCourierInput {
            id: Uuid::now_v7(),
            name: "Reddeden Kurye".to_string(),
            vehicle: VehicleType::Motorcycle,
        })
        .await
        .expect("register courier");
    couriers
        .update_location(courier.id, Location::new(41.0, 29.0).unwrap())
        .await
        .expect("courier location");

    let order = orders.create(order_input()).await.expect("create order");
    dispatch
        .offer_for_order(order.id)
        .await
        .expect("offer")
        .expect("a candidate exists");

    dispatch
        .reject_offer(order.id, courier.id)
        .await
        .expect("reject offer");

    let order_after_reject = orders.get(order.id).await.expect("fetch order");
    assert_eq!(order_after_reject.status, OrderStatus::Pending);
    let courier_after_reject = couriers.get(courier.id).await.expect("fetch courier");
    assert_eq!(courier_after_reject.status, CourierStatus::Available);

    // Accepting the same (now-rejected) offer must fail.
    assert!(dispatch.accept_offer(order.id, courier.id).await.is_err());
}

#[tokio::test]
async fn a_courier_cannot_accept_or_reject_someone_elses_offer() {
    let store = InMemoryStore::new();
    let couriers = CourierService::new(store.courier_repository());
    let orders = OrderService::new(store.order_repository());
    let dispatch = DispatchService::new(
        store.order_repository(),
        store.courier_repository(),
        store.assignment_repository(),
    );

    let courier = couriers
        .register(RegisterCourierInput {
            id: Uuid::now_v7(),
            name: "Gercek Alici".to_string(),
            vehicle: VehicleType::Motorcycle,
        })
        .await
        .expect("register courier");
    couriers
        .update_location(courier.id, Location::new(41.0, 29.0).unwrap())
        .await
        .expect("courier location");

    let order = orders.create(order_input()).await.expect("create order");
    dispatch
        .offer_for_order(order.id)
        .await
        .expect("offer")
        .expect("a candidate exists");

    let stranger_id = Uuid::now_v7();
    assert!(dispatch.accept_offer(order.id, stranger_id).await.is_err());
    assert!(dispatch.reject_offer(order.id, stranger_id).await.is_err());
}

#[tokio::test]
async fn offering_with_no_available_courier_returns_none_and_keeps_order_pending() {
    let store = InMemoryStore::new();
    let orders = OrderService::new(store.order_repository());
    let dispatch = DispatchService::new(
        store.order_repository(),
        store.courier_repository(),
        store.assignment_repository(),
    );

    let order = orders.create(order_input()).await.expect("create order");
    let offer = dispatch.offer_for_order(order.id).await.expect("offer");
    assert!(offer.is_none());
    let still_pending = orders.get(order.id).await.expect("fetch order");
    assert_eq!(still_pending.status, OrderStatus::Pending);
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
