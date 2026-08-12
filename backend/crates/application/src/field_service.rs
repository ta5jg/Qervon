// =============================================================================
// File:           backend/crates/application/src/field_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   Field Service & Appointment Time-Slot Scheduling Engine.
//
// Specification:
//   QAS-000004, QES-000006.
// =============================================================================
// STATUS: v2 backlog -- domain model + unit tests only; no repository, migration, or HTTP route yet. See BACKEND_BACKLOG.md.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSlotWindow {
    Morning,   // 09:00 - 12:00
    Afternoon, // 12:00 - 16:00
    Evening,   // 16:00 - 20:00
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldServiceAppointment {
    pub id: uuid::Uuid,
    pub customer_id: uuid::Uuid,
    pub technician_id: Option<uuid::Uuid>,
    pub service_type: String, // e.g. "Maintenance", "Installation", "ScheduledDelivery"
    pub appointment_date: String, // YYYY-MM-DD
    pub slot_window: TimeSlotWindow,
    pub is_confirmed: bool,
}

pub struct FieldServiceScheduler;

impl FieldServiceScheduler {
    pub fn schedule_appointment(
        customer_id: uuid::Uuid,
        service_type: impl Into<String>,
        date: impl Into<String>,
        slot: TimeSlotWindow,
    ) -> FieldServiceAppointment {
        FieldServiceAppointment {
            id: uuid::Uuid::now_v7(),
            customer_id,
            technician_id: None,
            service_type: service_type.into(),
            appointment_date: date.into(),
            slot_window: slot,
            is_confirmed: true,
        }
    }

    pub fn assign_technician(appointment: &mut FieldServiceAppointment, technician_id: uuid::Uuid) {
        appointment.technician_id = Some(technician_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schedules_and_assigns_technician() {
        let cust_id = uuid::Uuid::now_v7();
        let mut appt = FieldServiceScheduler::schedule_appointment(
            cust_id,
            "Klima Bakımı & Montaj",
            "2026-08-10",
            TimeSlotWindow::Morning,
        );

        assert!(appt.is_confirmed);
        assert!(appt.technician_id.is_none());

        let tech_id = uuid::Uuid::now_v7();
        FieldServiceScheduler::assign_technician(&mut appt, tech_id);

        assert_eq!(appt.technician_id, Some(tech_id));
    }
}
