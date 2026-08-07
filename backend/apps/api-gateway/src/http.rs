// =============================================================================
// File:           backend/apps/api-gateway/src/http.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   HTTP router and handlers for the Qervon delivery vertical slice.
//
// Specification:
//   QAS-000005, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use qervon_api_contracts::{
    AddressDto, AssignCourierRequest, CourierResponse, CreateOrderRequest, OrderResponse,
    RegisterCourierRequest, UpdateLocationRequest,
};
use qervon_application::{CreateOrderInput, RegisterCourierInput};
use qervon_domain::{Address, Location, Money, OrderId, VehicleType};
use serde_json::{json, Value};

use crate::api_error::ApiError;
use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(serve_dashboard))
        .route("/customer", get(serve_customer_portal))
        .route("/health", get(health))
        .route("/v1/users", post(register_user))
        .route("/v1/couriers", post(register_courier).get(list_couriers))
        .route("/v1/couriers/{id}/location", post(update_courier_location))
        .route("/v1/orders", post(create_order))
        .route("/v1/orders/{id}", get(get_order))
        .route("/v1/orders/{id}/assign", post(assign_courier))
        .route("/v1/orders/{id}/transit", post(start_transit))
        .route("/v1/orders/{id}/deliver", post(deliver_order))
        .route("/v1/orders/{id}/cancel", post(cancel_order))
        .route("/ws/tracking", get(ws_tracking_handler))
        .with_state(state)
}

async fn serve_dashboard() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../static/index.html"))
}

async fn serve_customer_portal() -> axum::response::Html<&'static str> {
    axum::response::Html(include_str!("../static/customer.html"))
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn ws_tracking_handler(
    ws: axum::extract::ws::WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl axum::response::IntoResponse {
    ws.on_upgrade(|mut socket| async move {
        let mut rx = state.location_tx.subscribe();
        while let Ok(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if socket.send(axum::extract::ws::Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    })
}

async fn register_user(
    State(state): State<AppState>,
    Json(request): Json<qervon_api_contracts::RegisterUserRequest>,
) -> Result<(StatusCode, Json<qervon_api_contracts::UserResponse>), ApiError> {
    let role: qervon_domain::UserRole = request
        .role
        .parse()
        .map_err(|_| ApiError::unprocessable("invalid user role"))?;
    let user = state
        .identity
        .register_user(qervon_application::CreateUserInput {
            email: request.email,
            display_name: request.display_name,
            role,
        })
        .await?;
    Ok((StatusCode::CREATED, Json((&user).into())))
}

async fn register_courier(
    State(state): State<AppState>,
    Json(request): Json<RegisterCourierRequest>,
) -> Result<(StatusCode, Json<CourierResponse>), ApiError> {
    let vehicle: VehicleType = request
        .vehicle
        .parse()
        .map_err(|_| ApiError::unprocessable("invalid vehicle type"))?;
    let courier = state
        .couriers
        .register_courier(RegisterCourierInput {
            id: request.id.unwrap_or_else(uuid::Uuid::now_v7),
            name: request.name,
            vehicle,
        })
        .await?;
    Ok((StatusCode::CREATED, Json((&courier).into())))
}

async fn list_couriers(
    State(state): State<AppState>,
) -> Result<Json<Vec<CourierResponse>>, ApiError> {
    let couriers = state.couriers.list_available_couriers().await?;
    Ok(Json(couriers.iter().map(CourierResponse::from).collect()))
}

async fn update_courier_location(
    State(state): State<AppState>,
    Path(courier_id): Path<uuid::Uuid>,
    Json(request): Json<UpdateLocationRequest>,
) -> Result<Json<CourierResponse>, ApiError> {
    let location = Location::new(request.latitude, request.longitude)?;
    let courier = state
        .couriers
        .update_courier_location(courier_id, location)
        .await?;
    
    // Broadcast live location event over WebSocket channel
    let _ = state.location_tx.send(crate::state::LocationUpdateEvent {
        courier_id,
        latitude: request.latitude,
        longitude: request.longitude,
        timestamp: chrono::Utc::now(),
    });

    Ok(Json((&courier).into()))
}

async fn create_order(
    State(state): State<AppState>,
    Json(request): Json<CreateOrderRequest>,
) -> Result<(StatusCode, Json<OrderResponse>), ApiError> {
    let order = state
        .orders
        .create_order(CreateOrderInput {
            customer_id: request.customer_id,
            pickup: to_address(request.pickup)?,
            dropoff: to_address(request.dropoff)?,
            fare: Money::new(request.fare_amount_minor, request.fare_currency)?,
        })
        .await?;
    Ok((StatusCode::CREATED, Json((&order).into())))
}

async fn get_order(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
) -> Result<Json<OrderResponse>, ApiError> {
    let order = state.orders.get_order(OrderId(order_id)).await?;
    Ok(Json((&order).into()))
}

async fn assign_courier(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
    Json(request): Json<AssignCourierRequest>,
) -> Result<(StatusCode, Json<qervon_api_contracts::AssignmentResponse>), ApiError> {
    let order_id = OrderId(order_id);
    let assignment = match request.courier_id {
        Some(courier_id) => state.dispatch.assign_courier(order_id, courier_id).await?,
        None => state.dispatch.auto_assign(order_id).await?,
    };
    Ok((StatusCode::OK, Json((&assignment).into())))
}

async fn start_transit(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
) -> Result<Json<OrderResponse>, ApiError> {
    let order = state.dispatch.start_transit(OrderId(order_id)).await?;
    Ok(Json((&order).into()))
}

async fn deliver_order(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
) -> Result<Json<OrderResponse>, ApiError> {
    let order = state.dispatch.deliver_order(OrderId(order_id)).await?;
    Ok(Json((&order).into()))
}

async fn cancel_order(
    State(state): State<AppState>,
    Path(order_id): Path<uuid::Uuid>,
) -> Result<Json<OrderResponse>, ApiError> {
    let order = state.dispatch.cancel_order(OrderId(order_id)).await?;
    Ok(Json((&order).into()))
}

fn to_address(dto: AddressDto) -> Result<Address, ApiError> {
    Ok(Address {
        location: Location::new(dto.latitude, dto.longitude)?,
        label: dto.label,
    })
}
