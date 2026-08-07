// =============================================================================
// File:           foundation/crates/events/src/event.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Domain event traits, envelope, and bus abstraction.
//
// Specification:
//   QAS-000003, QFS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ---------------------------------------------------------------------------
// EventEnvelope — wraps any domain event with metadata
// ---------------------------------------------------------------------------

/// A transport-agnostic envelope that carries a domain event together with
/// routing metadata.  The `payload` field holds the serialised event body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventEnvelope {
    /// Unique ID for this particular event occurrence.
    pub id: Uuid,
    /// Machine-readable event type, e.g. `"order.created"`.
    pub event_type: String,
    /// ID of the aggregate that produced the event.
    pub aggregate_id: Uuid,
    /// Human-readable aggregate kind, e.g. `"order"`, `"courier"`.
    pub aggregate_type: String,
    /// When the domain event occurred.
    pub occurred_at: DateTime<Utc>,
    /// JSON-serialised event body.
    pub payload: serde_json::Value,
}

impl EventEnvelope {
    /// Construct a new envelope by serialising `payload` as JSON.
    pub fn new<T: Serialize>(
        event_type: impl Into<String>,
        aggregate_id: Uuid,
        aggregate_type: impl Into<String>,
        occurred_at: DateTime<Utc>,
        payload: &T,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self {
            id: Uuid::now_v7(),
            event_type: event_type.into(),
            aggregate_id,
            aggregate_type: aggregate_type.into(),
            occurred_at,
            payload: serde_json::to_value(payload)?,
        })
    }
}

// ---------------------------------------------------------------------------
// EventBus — publish / subscribe abstraction
// ---------------------------------------------------------------------------

/// Async event bus port.  Infrastructure adapters (in-memory, Redis Pub/Sub,
/// Kafka, etc.) implement this trait.
#[async_trait]
pub trait EventBus: Send + Sync {
    /// Publish a single event envelope.
    async fn publish(&self, event: &EventEnvelope) -> Result<(), EventBusError>;
}

/// Errors that an `EventBus` adapter may return.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EventBusError {
    #[error("serialization failed: {0}")]
    Serialization(String),
    #[error("transport failed: {0}")]
    Transport(String),
}

// ---------------------------------------------------------------------------
// Concrete domain events
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderCreated {
    pub order_id: Uuid,
    pub customer_id: Uuid,
    pub pickup_lat: f64,
    pub pickup_lng: f64,
    pub dropoff_lat: f64,
    pub dropoff_lng: f64,
    pub fare_minor: i64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDelivered {
    pub order_id: Uuid,
    pub courier_id: Uuid,
    pub delivered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourierAssigned {
    pub order_id: Uuid,
    pub courier_id: Uuid,
    pub assignment_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourierLocationUpdated {
    pub courier_id: Uuid,
    pub latitude: f64,
    pub longitude: f64,
    pub speed_kmh: Option<f64>,
    pub battery_pct: Option<u8>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceIssued {
    pub invoice_id: Uuid,
    pub order_id: Uuid,
    pub customer_id: Uuid,
    pub amount_minor: i64,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSent {
    pub notification_id: Uuid,
    pub recipient_id: Uuid,
    pub channel: String,
    pub sent_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// InMemoryEventBus — for tests and local development
// ---------------------------------------------------------------------------

use std::sync::{Arc, Mutex};

/// A simple in-memory event bus for testing and local development.
/// Stores all published events in a `Vec`.
#[derive(Debug, Clone, Default)]
pub struct InMemoryEventBus {
    events: Arc<Mutex<Vec<EventEnvelope>>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn events(&self) -> Vec<EventEnvelope> {
        self.events.lock().expect("lock poisoned").clone()
    }

    pub fn len(&self) -> usize {
        self.events.lock().expect("lock poisoned").len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish(&self, event: &EventEnvelope) -> Result<(), EventBusError> {
        self.events
            .lock()
            .map_err(|e| EventBusError::Transport(e.to_string()))?
            .push(event.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn envelope_serialises_payload_as_json() {
        let evt = OrderCreated {
            order_id: Uuid::now_v7(),
            customer_id: Uuid::now_v7(),
            pickup_lat: 41.0,
            pickup_lng: 29.0,
            dropoff_lat: 41.1,
            dropoff_lng: 29.1,
            fare_minor: 5_000,
            currency: "TRY".to_string(),
        };
        let envelope = EventEnvelope::new(
            "order.created",
            evt.order_id,
            "order",
            Utc::now(),
            &evt,
        )
        .expect("serialise");
        assert_eq!(envelope.event_type, "order.created");
        assert_eq!(envelope.aggregate_type, "order");
        assert_eq!(envelope.payload["fare_minor"], 5_000);
    }

    #[tokio::test]
    async fn in_memory_bus_stores_events() {
        let bus = InMemoryEventBus::new();
        assert!(bus.is_empty());

        let envelope = EventEnvelope::new(
            "courier.assigned",
            Uuid::now_v7(),
            "order",
            Utc::now(),
            &CourierAssigned {
                order_id: Uuid::now_v7(),
                courier_id: Uuid::now_v7(),
                assignment_id: Uuid::now_v7(),
            },
        )
        .expect("serialise");

        bus.publish(&envelope).await.expect("publish");
        assert_eq!(bus.len(), 1);
        assert_eq!(bus.events()[0].event_type, "courier.assigned");
    }
}
