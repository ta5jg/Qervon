// =============================================================================
// File:           backend/crates/domain/src/field_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.2.0
//
// Description:
//   Field Service & Appointment Time-Slot Scheduling Engine. Moved from
//   qervon-application to qervon-domain so its repository trait can live
//   alongside every other repository port in repository.rs (a domain crate
//   cannot depend on the application crate that used to own this model).
//
// Specification:
//   QAS-000004, QES-000006.
// =============================================================================
// STATUS: wired -- Postgres-backed repository (FieldServiceAppointmentRepository),
// a governed migration adding tenant_id, and tenant-scoped HTTP routes are all
// wired in api-gateway. See BACKEND_BACKLOG.md for history.

use crate::tenant::TenantId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSlotWindow {
    Morning,   // 09:00 - 12:00
    Afternoon, // 12:00 - 16:00
    Evening,   // 16:00 - 20:00
}

impl TimeSlotWindow {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Morning => "Morning",
            Self::Afternoon => "Afternoon",
            Self::Evening => "Evening",
        }
    }
}

impl std::str::FromStr for TimeSlotWindow {
    type Err = crate::DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "Morning" => Ok(Self::Morning),
            "Afternoon" => Ok(Self::Afternoon),
            "Evening" => Ok(Self::Evening),
            other => Err(crate::DomainError::validation(format!(
                "unknown time slot window: {other}"
            ))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldServiceAppointment {
    pub id: uuid::Uuid,
    pub tenant_id: TenantId,
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
        tenant_id: TenantId,
        customer_id: uuid::Uuid,
        service_type: impl Into<String>,
        date: impl Into<String>,
        slot: TimeSlotWindow,
    ) -> FieldServiceAppointment {
        FieldServiceAppointment {
            id: uuid::Uuid::now_v7(),
            tenant_id,
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
            TenantId::new(),
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

    #[test]
    fn time_slot_window_string_round_trip() {
        for variant in [
            TimeSlotWindow::Morning,
            TimeSlotWindow::Afternoon,
            TimeSlotWindow::Evening,
        ] {
            assert_eq!(variant.as_str().parse::<TimeSlotWindow>(), Ok(variant));
        }
    }
}
