// =============================================================================
// File:           backend/crates/application/src/courier_leaderboard.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   Courier Gamification & Weekly Performance Leaderboard Engine.
//
// Specification:
//   QAS-000004, QES-000006.
// =============================================================================
// STATUS: wired -- exposed as a tenant-scoped read model at
// GET /v1/couriers/leaderboard in api-gateway. It has no repository or
// migration of its own on purpose: every input (completed deliveries,
// on-time rate, average rating) is derived live from the existing Order and
// CustomerRating repositories rather than duplicated into a new table. See
// BACKEND_BACKLOG.md for history.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourierLeaderboardEntry {
    pub courier_id: uuid::Uuid,
    pub courier_name: String,
    pub completed_deliveries: u32,
    pub on_time_rate_percent: f64,
    pub average_rating: f64,
    pub total_score: f64,
    pub rank: u32,
}

pub struct CourierLeaderboardEngine;

impl CourierLeaderboardEngine {
    /// Calculate composite performance score and rank couriers
    pub fn calculate_leaderboard(
        mut entries: Vec<CourierLeaderboardEntry>,
    ) -> Vec<CourierLeaderboardEntry> {
        for entry in entries.iter_mut() {
            // Formula: (Completed * 10) + (OnTime% * 5) + (Rating * 50)
            entry.total_score = (entry.completed_deliveries as f64 * 10.0)
                + (entry.on_time_rate_percent * 5.0)
                + (entry.average_rating * 50.0);
        }

        // Sort descending by total score
        entries.sort_by(|a, b| b.total_score.total_cmp(&a.total_score));

        // Assign ranks
        for (idx, entry) in entries.iter_mut().enumerate() {
            entry.rank = (idx + 1) as u32;
        }

        entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_high_performing_courier_first() {
        let c1 = CourierLeaderboardEntry {
            courier_id: uuid::Uuid::now_v7(),
            courier_name: "Ahmet".into(),
            completed_deliveries: 50,
            on_time_rate_percent: 98.0,
            average_rating: 4.9,
            total_score: 0.0,
            rank: 0,
        };

        let c2 = CourierLeaderboardEntry {
            courier_id: uuid::Uuid::now_v7(),
            courier_name: "Mehmet".into(),
            completed_deliveries: 20,
            on_time_rate_percent: 85.0,
            average_rating: 4.2,
            total_score: 0.0,
            rank: 0,
        };

        let ranked = CourierLeaderboardEngine::calculate_leaderboard(vec![c2, c1.clone()]);
        assert_eq!(ranked[0].courier_name, "Ahmet");
        assert_eq!(ranked[0].rank, 1);
    }
}
