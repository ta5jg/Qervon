// =============================================================================
// File:           backend/crates/domain/src/proof_of_delivery.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   Proof of Delivery (POD) Digital Signature & QR Verification Model.
//
// Specification:
//   QAS-000004, QES-000006.
// =============================================================================

use crate::DomainError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofOfDeliveryRecord {
    pub id: uuid::Uuid,
    pub order_id: uuid::Uuid,
    pub courier_id: uuid::Uuid,
    pub recipient_name: String,
    pub qr_barcode_verified: bool,
    pub digital_signature_base64: Option<String>,
    pub photo_evidence_url: Option<String>,
    pub delivered_at: chrono::DateTime<chrono::Utc>,
}

impl ProofOfDeliveryRecord {
    pub fn new(
        order_id: uuid::Uuid,
        courier_id: uuid::Uuid,
        recipient_name: impl Into<String>,
        qr_barcode_verified: bool,
        digital_signature_base64: Option<String>,
        photo_evidence_url: Option<String>,
    ) -> Result<Self, DomainError> {
        let recipient = recipient_name.into();
        if recipient.trim().is_empty() {
            return Err(DomainError::Validation("Recipient name cannot be empty".into()));
        }

        Ok(Self {
            id: uuid::Uuid::now_v7(),
            order_id,
            courier_id,
            recipient_name: recipient,
            qr_barcode_verified,
            digital_signature_base64,
            photo_evidence_url,
            delivered_at: chrono::Utc::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_pod_creation() {
        let order_id = uuid::Uuid::now_v7();
        let courier_id = uuid::Uuid::now_v7();

        let pod = ProofOfDeliveryRecord::new(
            order_id,
            courier_id,
            "Ali Yılmaz",
            true,
            Some("data:image/png;base64,iVBORw...".into()),
            Some("https://storage.qervon.com/pod/123.jpg".into()),
        ).unwrap();

        assert_eq!(pod.recipient_name, "Ali Yılmaz");
        assert!(pod.qr_barcode_verified);
    }
}
