// =============================================================================
// File:           backend/crates/application/src/tax_invoicing.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   Tax Compliance, Auto E-Invoicing & Financial Revenue Engine.
//
// Specification:
//   QAS-000006, QES-000006.
// =============================================================================
// STATUS: v2 backlog -- domain model + unit tests only; no repository, migration, or HTTP route yet. See BACKEND_BACKLOG.md.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectronicInvoiceDraft {
    pub invoice_number: String,
    pub order_id: uuid::Uuid,
    pub customer_id: uuid::Uuid,
    pub net_amount_minor: i64,
    pub vat_amount_minor: i64,
    pub total_amount_minor: i64,
    pub vat_rate_percent: f64,
    pub currency: String,
    pub issue_date: chrono::DateTime<chrono::Utc>,
}

pub struct TaxInvoicingEngine;

impl TaxInvoicingEngine {
    /// Generate e-Invoice draft with 20% VAT compliance
    pub fn generate_e_invoice(
        order_id: uuid::Uuid,
        customer_id: uuid::Uuid,
        net_amount_minor: i64,
        currency: impl Into<String>,
    ) -> ElectronicInvoiceDraft {
        let vat_rate_percent = 20.0;
        let vat_amount_minor = ((net_amount_minor as f64) * (vat_rate_percent / 100.0)) as i64;
        let total_amount_minor = net_amount_minor + vat_amount_minor;

        let inv_suffix = uuid::Uuid::now_v7().to_string();
        let invoice_number = format!("QER2026{}", inv_suffix[..8].to_uppercase());

        ElectronicInvoiceDraft {
            invoice_number,
            order_id,
            customer_id,
            net_amount_minor,
            vat_amount_minor,
            total_amount_minor,
            vat_rate_percent,
            currency: currency.into(),
            issue_date: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_20_percent_vat_and_total() {
        let order_id = uuid::Uuid::now_v7();
        let cust_id = uuid::Uuid::now_v7();

        let inv = TaxInvoicingEngine::generate_e_invoice(order_id, cust_id, 10000, "TRY"); // Net: ₺100.00

        assert_eq!(inv.net_amount_minor, 10000);
        assert_eq!(inv.vat_amount_minor, 2000); // KDV %20: ₺20.00
        assert_eq!(inv.total_amount_minor, 12000); // Toplam: ₺120.00
        assert!(inv.invoice_number.starts_with("QER2026"));
    }
}
