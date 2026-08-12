// =============================================================================
// File:           backend/modules/billing/src/lib.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Billing domain module: public boundary over invoice and payout use cases.
//
// Specification:
//   QLS-000009, QAS-000002, QAS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use qervon_application::{BillingService, CreateInvoiceInput, CreatePayoutInput};
use qervon_domain::{
    CourierPayout, CourierPayoutRepository, Invoice, InvoiceId, InvoiceRepository,
};
use uuid::Uuid;

pub struct BillingModule<IR, PR>
where
    IR: InvoiceRepository,
    PR: CourierPayoutRepository,
{
    service: BillingService<IR, PR>,
}

impl<IR, PR> BillingModule<IR, PR>
where
    IR: InvoiceRepository,
    PR: CourierPayoutRepository,
{
    pub fn new(invoices: IR, payouts: PR) -> Self {
        Self {
            service: BillingService::new(invoices, payouts),
        }
    }

    pub async fn create_invoice(
        &self,
        input: CreateInvoiceInput,
    ) -> Result<Invoice, qervon_application::ApplicationError> {
        self.service.create_invoice(input).await
    }

    pub async fn get_invoice(
        &self,
        id: InvoiceId,
    ) -> Result<Invoice, qervon_application::ApplicationError> {
        self.service.get_invoice(id).await
    }

    pub async fn find_invoice_for_order(
        &self,
        order_id: qervon_domain::OrderId,
    ) -> Result<Option<Invoice>, qervon_application::ApplicationError> {
        self.service.find_invoice_for_order(order_id).await
    }

    pub async fn issue_invoice(
        &self,
        id: InvoiceId,
    ) -> Result<Invoice, qervon_application::ApplicationError> {
        self.service.issue_invoice(id).await
    }

    pub async fn pay_invoice(
        &self,
        id: InvoiceId,
    ) -> Result<Invoice, qervon_application::ApplicationError> {
        self.service.pay_invoice(id).await
    }

    pub async fn create_payout(
        &self,
        input: CreatePayoutInput,
    ) -> Result<CourierPayout, qervon_application::ApplicationError> {
        self.service.create_payout(input).await
    }

    pub async fn list_payouts_for_courier(
        &self,
        courier_id: Uuid,
    ) -> Result<Vec<CourierPayout>, qervon_application::ApplicationError> {
        self.service.list_payouts_for_courier(courier_id).await
    }
}
