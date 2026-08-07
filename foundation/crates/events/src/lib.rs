/* =============================================================================
 * File:           foundation/crates/events/src/lib.rs
 * Project:        Qervon
 * Author:         USDTG GROUP TECHNOLOGY LLC
 * Developer:      Irfan Gedik
 * Created Date:   2026-08-05
 * Version:        0.1.0
 *
 * Description:
 *   Qervon Events foundation crate: domain event envelope, bus abstraction,
 *   concrete event types, and an in-memory bus for testing.
 *
 * Specification:
 *   QAS-000003, QFS-000003, QES-000002, QES-000006.
 *
 * License:
 *   Qervon License v1.0 — see LICENSE in the repository root.
 * ============================================================================= */

pub mod event;

pub use event::{
    // Core abstractions
    EventBus,
    EventBusError,
    EventEnvelope,
    // Concrete events
    CourierAssigned,
    CourierLocationUpdated,
    InvoiceIssued,
    NotificationSent,
    OrderCreated,
    OrderDelivered,
    // In-memory adapter
    InMemoryEventBus,
};
