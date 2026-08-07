// =============================================================================
// File:           backend/crates/application/src/billing_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Billing use cases: invoice lifecycle, courier payouts.
//
// Specification:
//   QLS-000009, QAS-000002, QAS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use qervon_domain::{
    CourierPayout, CourierPayoutRepository, Invoice, InvoiceId, InvoiceRepository, Money, OrderId,
};
use uuid::Uuid;

use crate::error::ApplicationError;

#[derive(Debug, Clone)]
pub struct CreateInvoiceInput {
    pub order_id: OrderId,
    pub customer_id: Uuid,
    pub amount: Money,
}

#[derive(Debug, Clone)]
pub struct CreatePayoutInput {
    pub courier_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub gross_amount: Money,
    pub commission: Money,
}

pub struct BillingService<IR, PR>
where
    IR: InvoiceRepository,
    PR: CourierPayoutRepository,
{
    invoices: IR,
    payouts: PR,
}

impl<IR, PR> BillingService<IR, PR>
where
    IR: InvoiceRepository,
    PR: CourierPayoutRepository,
{
    pub fn new(invoices: IR, payouts: PR) -> Self {
        Self { invoices, payouts }
    }

    // ---- Invoice operations ----

    pub async fn create_invoice(
        &self,
        input: CreateInvoiceInput,
    ) -> Result<Invoice, ApplicationError> {
        let invoice = Invoice::create(
            InvoiceId::new(),
            input.order_id,
            input.customer_id,
            input.amount,
            Utc::now(),
        )?;
        self.invoices.create(&invoice).await?;
        Ok(invoice)
    }

    pub async fn get_invoice(&self, id: InvoiceId) -> Result<Invoice, ApplicationError> {
        self.invoices
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }

    pub async fn issue_invoice(&self, id: InvoiceId) -> Result<Invoice, ApplicationError> {
        let mut invoice = self.get_invoice(id).await?;
        invoice.issue(Utc::now())?;
        self.invoices.update(&invoice).await?;
        Ok(invoice)
    }

    pub async fn pay_invoice(&self, id: InvoiceId) -> Result<Invoice, ApplicationError> {
        let mut invoice = self.get_invoice(id).await?;
        invoice.pay(Utc::now())?;
        self.invoices.update(&invoice).await?;
        Ok(invoice)
    }

    pub async fn cancel_invoice(&self, id: InvoiceId) -> Result<Invoice, ApplicationError> {
        let mut invoice = self.get_invoice(id).await?;
        invoice.cancel()?;
        self.invoices.update(&invoice).await?;
        Ok(invoice)
    }

    pub async fn refund_invoice(&self, id: InvoiceId) -> Result<Invoice, ApplicationError> {
        let mut invoice = self.get_invoice(id).await?;
        invoice.refund()?;
        self.invoices.update(&invoice).await?;
        Ok(invoice)
    }

    // ---- Payout operations ----

    pub async fn create_payout(
        &self,
        input: CreatePayoutInput,
    ) -> Result<CourierPayout, ApplicationError> {
        let payout = CourierPayout::create(
            input.courier_id,
            input.period_start,
            input.period_end,
            input.gross_amount,
            input.commission,
            Utc::now(),
        )?;
        self.payouts.create(&payout).await?;
        Ok(payout)
    }

    pub async fn list_payouts_for_courier(
        &self,
        courier_id: Uuid,
    ) -> Result<Vec<CourierPayout>, ApplicationError> {
        Ok(self.payouts.find_by_courier(courier_id).await?)
    }
}
