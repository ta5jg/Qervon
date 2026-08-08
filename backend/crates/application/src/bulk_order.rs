// =============================================================================
// File:           backend/crates/application/src/bulk_order.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   Bulk CSV order ingestion parser and Webhook Notification Data Structures.
//
// Specification:
//   QAS-000005, QES-000006.
// =============================================================================

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkOrderRow {
    pub customer_id: uuid::Uuid,
    pub pickup_label: String,
    pub pickup_lat: f64,
    pub pickup_lon: f64,
    pub dropoff_label: String,
    pub dropoff_lat: f64,
    pub dropoff_lon: f64,
    pub fare_amount_minor: i64,
    pub currency: String,
}

pub struct BulkOrderParser;

impl BulkOrderParser {
    /// Parse simple CSV text line by line into BulkOrderRow structures
    pub fn parse_csv(csv_content: &str) -> Result<Vec<BulkOrderRow>, String> {
        let mut rows = Vec::new();
        for (idx, line) in csv_content.lines().enumerate() {
            if idx == 0 || line.trim().is_empty() {
                continue; // Skip header or empty lines
            }
            let parts: Vec<&str> = line.split(',').collect();
            if parts.len() >= 9 {
                let customer_id = uuid::Uuid::parse_str(parts[0].trim())
                    .map_err(|e| format!("Line {}: Invalid UUID {}", idx + 1, e))?;
                let fare_amount_minor = parts[7].trim().parse::<i64>()
                    .map_err(|e| format!("Line {}: Invalid Fare {}", idx + 1, e))?;

                rows.push(BulkOrderRow {
                    customer_id,
                    pickup_label: parts[1].trim().to_string(),
                    pickup_lat: parts[2].trim().parse().unwrap_or(0.0),
                    pickup_lon: parts[3].trim().parse().unwrap_or(0.0),
                    dropoff_label: parts[4].trim().to_string(),
                    dropoff_lat: parts[5].trim().parse().unwrap_or(0.0),
                    dropoff_lon: parts[6].trim().parse().unwrap_or(0.0),
                    fare_amount_minor,
                    currency: parts[8].trim().to_string(),
                });
            }
        }
        Ok(rows)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebhookPayload {
    pub event_type: String,
    pub order_id: uuid::Uuid,
    pub status: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_bulk_order_csv() {
        let csv_data = "customer_id,pickup_label,pickup_lat,pickup_lon,dropoff_label,dropoff_lat,dropoff_lon,fare_amount_minor,currency\n\
        00000000-0000-0000-0000-000000000001,Pickup Point,41.0,29.0,Dropoff Point,41.1,29.1,4500,TRY\n";

        let parsed = BulkOrderParser::parse_csv(csv_data).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].fare_amount_minor, 4500);
        assert_eq!(parsed[0].currency, "TRY");
    }
}
