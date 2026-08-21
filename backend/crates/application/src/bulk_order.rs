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

use std::collections::HashSet;

use csv::Trim;
use serde::{Deserialize, Serialize};

pub const MAX_BULK_ORDER_ROWS: usize = 100;

const REQUIRED_HEADERS: [&str; 8] = [
    "reference",
    "pickup_label",
    "pickup_latitude",
    "pickup_longitude",
    "dropoff_label",
    "dropoff_latitude",
    "dropoff_longitude",
    "contact_phone",
];

const ALLOWED_HEADERS: [&str; 10] = [
    "reference",
    "pickup_label",
    "pickup_latitude",
    "pickup_longitude",
    "dropoff_label",
    "dropoff_latitude",
    "dropoff_longitude",
    "contact_phone",
    "payment_method",
    "delivery_note",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BulkOrderRow {
    /// Customer-controlled reconciliation key. It must be unique inside one
    /// file, but never decides ownership; the authenticated session does.
    pub reference: String,
    pub pickup_label: String,
    pub pickup_latitude: f64,
    pub pickup_longitude: f64,
    pub dropoff_label: String,
    pub dropoff_latitude: f64,
    pub dropoff_longitude: f64,
    pub contact_phone: String,
    #[serde(default)]
    pub payment_method: Option<String>,
    #[serde(default)]
    pub delivery_note: Option<String>,
}

pub struct BulkOrderParser;

impl BulkOrderParser {
    /// Parses an RFC 4180 CSV file and validates every row before returning.
    ///
    /// Ownership and fares are deliberately absent from this contract. The
    /// API derives the customer/tenant from its authenticated session and
    /// computes each fare from tenant pricing on the server.
    pub fn parse_csv(csv_content: &str) -> Result<Vec<BulkOrderRow>, String> {
        let mut reader = csv::ReaderBuilder::new()
            .trim(Trim::All)
            .flexible(false)
            .from_reader(csv_content.as_bytes());
        let headers = reader
            .headers()
            .map_err(|error| format!("CSV başlığı okunamadı: {error}"))?
            .clone();
        let supplied: HashSet<&str> = headers.iter().collect();

        for required in REQUIRED_HEADERS {
            if !supplied.contains(required) {
                return Err(format!("Zorunlu CSV sütunu eksik: {required}"));
            }
        }
        for header in &headers {
            if !ALLOWED_HEADERS.contains(&header) {
                return Err(format!("Bilinmeyen CSV sütunu: {header}"));
            }
        }

        let mut rows = Vec::new();
        let mut references = HashSet::new();
        for (index, record) in reader.deserialize::<BulkOrderRow>().enumerate() {
            let line = index + 2;
            if rows.len() == MAX_BULK_ORDER_ROWS {
                return Err(format!(
                    "Bir dosyada en fazla {MAX_BULK_ORDER_ROWS} sipariş olabilir"
                ));
            }
            let mut row = record.map_err(|error| format!("Satır {line}: {error}"))?;
            row.reference = row.reference.trim().to_owned();
            row.pickup_label = row.pickup_label.trim().to_owned();
            row.dropoff_label = row.dropoff_label.trim().to_owned();
            row.contact_phone = row.contact_phone.trim().to_owned();
            row.payment_method = row
                .payment_method
                .map(|value| value.trim().to_ascii_lowercase())
                .filter(|value| !value.is_empty());
            row.delivery_note = row
                .delivery_note
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty());

            Self::validate_row(&row, line)?;
            if !references.insert(row.reference.clone()) {
                return Err(format!(
                    "Satır {line}: reference değeri dosya içinde benzersiz olmalıdır"
                ));
            }
            rows.push(row);
        }
        if rows.is_empty() {
            return Err("CSV dosyasında sipariş satırı bulunamadı".to_owned());
        }
        Ok(rows)
    }

    fn validate_row(row: &BulkOrderRow, line: usize) -> Result<(), String> {
        if row.reference.is_empty() || row.reference.chars().count() > 64 {
            return Err(format!(
                "Satır {line}: reference 1-64 karakter arasında olmalıdır"
            ));
        }
        for (name, value) in [
            ("pickup_label", row.pickup_label.as_str()),
            ("dropoff_label", row.dropoff_label.as_str()),
        ] {
            if value.is_empty() || value.chars().count() > 300 {
                return Err(format!(
                    "Satır {line}: {name} 1-300 karakter arasında olmalıdır"
                ));
            }
        }
        Self::validate_coordinate(row.pickup_latitude, -90.0, 90.0, "pickup_latitude", line)?;
        Self::validate_coordinate(
            row.pickup_longitude,
            -180.0,
            180.0,
            "pickup_longitude",
            line,
        )?;
        Self::validate_coordinate(row.dropoff_latitude, -90.0, 90.0, "dropoff_latitude", line)?;
        Self::validate_coordinate(
            row.dropoff_longitude,
            -180.0,
            180.0,
            "dropoff_longitude",
            line,
        )?;
        if row
            .contact_phone
            .chars()
            .filter(char::is_ascii_digit)
            .count()
            < 10
        {
            return Err(format!(
                "Satır {line}: geçerli bir contact_phone zorunludur"
            ));
        }
        if let Some(method) = row.payment_method.as_deref() {
            if !matches!(method, "cash" | "card" | "wallet") {
                return Err(format!(
                    "Satır {line}: payment_method cash, card veya wallet olmalıdır"
                ));
            }
        }
        Ok(())
    }

    fn validate_coordinate(
        value: f64,
        minimum: f64,
        maximum: f64,
        name: &str,
        line: usize,
    ) -> Result<(), String> {
        if !value.is_finite() || !(minimum..=maximum).contains(&value) {
            return Err(format!(
                "Satır {line}: {name} {minimum} ile {maximum} arasında olmalıdır"
            ));
        }
        Ok(())
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
        let csv_data = "reference,pickup_label,pickup_latitude,pickup_longitude,dropoff_label,dropoff_latitude,dropoff_longitude,contact_phone,payment_method,delivery_note\n\
        ORD-001,\"Pickup, Point\",41.0,29.0,Dropoff Point,41.1,29.1,05550000000,CARD,Kapıcıya bırakın\n";

        let parsed = BulkOrderParser::parse_csv(csv_data).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].reference, "ORD-001");
        assert_eq!(parsed[0].pickup_label, "Pickup, Point");
        assert_eq!(parsed[0].payment_method.as_deref(), Some("card"));
    }

    #[test]
    fn accepts_excel_compatible_utf8_bom() {
        let csv_data = "\u{feff}reference,pickup_label,pickup_latitude,pickup_longitude,dropoff_label,dropoff_latitude,dropoff_longitude,contact_phone\n\
        ORD-001,Pickup,41.0,29.0,Dropoff,41.1,29.1,05550000000\n";

        let parsed = BulkOrderParser::parse_csv(csv_data).unwrap();
        assert_eq!(parsed[0].reference, "ORD-001");
    }

    #[test]
    fn rejects_client_supplied_fare_columns() {
        let csv_data = "reference,pickup_label,pickup_latitude,pickup_longitude,dropoff_label,dropoff_latitude,dropoff_longitude,contact_phone,fare_amount_minor\n\
        ORD-001,Pickup,41.0,29.0,Dropoff,41.1,29.1,05550000000,1\n";

        let error = BulkOrderParser::parse_csv(csv_data).unwrap_err();
        assert!(error.contains("Bilinmeyen CSV sütunu: fare_amount_minor"));
    }

    #[test]
    fn rejects_duplicate_references_before_import() {
        let csv_data = "reference,pickup_label,pickup_latitude,pickup_longitude,dropoff_label,dropoff_latitude,dropoff_longitude,contact_phone\n\
        ORD-001,Pickup,41.0,29.0,Dropoff,41.1,29.1,05550000000\n\
        ORD-001,Pickup 2,41.2,29.2,Dropoff 2,41.3,29.3,05550000001\n";

        let error = BulkOrderParser::parse_csv(csv_data).unwrap_err();
        assert!(error.contains("dosya içinde benzersiz"));
    }

    #[test]
    fn rejects_invalid_coordinates_and_phone_numbers() {
        let invalid_coordinate = "reference,pickup_label,pickup_latitude,pickup_longitude,dropoff_label,dropoff_latitude,dropoff_longitude,contact_phone\n\
        ORD-001,Pickup,91,29.0,Dropoff,41.1,29.1,05550000000\n";
        assert!(BulkOrderParser::parse_csv(invalid_coordinate)
            .unwrap_err()
            .contains("pickup_latitude"));

        let invalid_phone = "reference,pickup_label,pickup_latitude,pickup_longitude,dropoff_label,dropoff_latitude,dropoff_longitude,contact_phone\n\
        ORD-001,Pickup,41.0,29.0,Dropoff,41.1,29.1,123\n";
        assert!(BulkOrderParser::parse_csv(invalid_phone)
            .unwrap_err()
            .contains("contact_phone"));
    }
}
