// =============================================================================
// File:           backend/crates/domain/src/courier_shift.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   Courier Shift & Weekly Roster Management Domain Model.
//
// Specification:
//   QAS-000002, QES-000006.
// =============================================================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShiftType {
    MorningShift,   // 08:00 - 16:00
    AfternoonShift, // 16:00 - 00:00
    NightShift,     // 00:00 - 08:00
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourierShiftAssignment {
    pub id: uuid::Uuid,
    pub courier_id: uuid::Uuid,
    pub shift_date: String, // YYYY-MM-DD
    pub shift_type: ShiftType,
    pub is_on_break: bool,
}

impl CourierShiftAssignment {
    pub fn new(courier_id: uuid::Uuid, shift_date: impl Into<String>, shift_type: ShiftType) -> Self {
        Self {
            id: uuid::Uuid::now_v7(),
            courier_id,
            shift_date: shift_date.into(),
            shift_type,
            is_on_break: false,
        }
    }

    pub fn set_break_status(&mut self, on_break: bool) {
        self.is_on_break = on_break;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_and_toggles_shift_break() {
        let courier_id = uuid::Uuid::now_v7();
        let mut shift = CourierShiftAssignment::new(courier_id, "2026-08-08", ShiftType::MorningShift);

        assert!(!shift.is_on_break);
        shift.set_break_status(true);
        assert!(shift.is_on_break);
    }
}
