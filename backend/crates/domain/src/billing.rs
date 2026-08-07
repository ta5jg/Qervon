// =============================================================================
// File:           backend/crates/domain/src/billing.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.1.0
//
// Description:
//   Billing domain: invoices for orders and courier payouts.
//
// Specification:
//   QLS-000009, QAS-000002, QAS-000003, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::DomainError;
use crate::money::Money;
use crate::order::OrderId;

// ---------------------------------------------------------------------------
// InvoiceId
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct InvoiceId(pub Uuid);

impl InvoiceId {
    pub fn new() -> Self {
        Self(Uuid::now_v7())
    }
}

impl Default for InvoiceId {
    fn default() -> Self {
        Self::new()
    }
}

// ---------------------------------------------------------------------------
// InvoiceStatus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Draft,
    Issued,
    Paid,
    Cancelled,
    Refunded,
}

impl InvoiceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Issued => "issued",
            Self::Paid => "paid",
            Self::Cancelled => "cancelled",
            Self::Refunded => "refunded",
        }
    }

    pub fn can_issue(&self) -> bool {
        matches!(self, Self::Draft)
    }

    pub fn can_pay(&self) -> bool {
        matches!(self, Self::Issued)
    }

    pub fn can_cancel(&self) -> bool {
        matches!(self, Self::Draft | Self::Issued)
    }

    pub fn can_refund(&self) -> bool {
        matches!(self, Self::Paid)
    }
}

impl std::str::FromStr for InvoiceStatus {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "draft" => Ok(Self::Draft),
            "issued" => Ok(Self::Issued),
            "paid" => Ok(Self::Paid),
            "cancelled" => Ok(Self::Cancelled),
            "refunded" => Ok(Self::Refunded),
            other => Err(DomainError::validation(format!(
                "unknown invoice status: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for InvoiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// Invoice entity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Invoice {
    pub id: InvoiceId,
    pub order_id: OrderId,
    pub customer_id: Uuid,
    pub amount: Money,
    pub status: InvoiceStatus,
    pub created_at: DateTime<Utc>,
    pub issued_at: Option<DateTime<Utc>>,
    pub paid_at: Option<DateTime<Utc>>,
}

impl Invoice {
    pub fn create(
        id: InvoiceId,
        order_id: OrderId,
        customer_id: Uuid,
        amount: Money,
        now: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if customer_id.is_nil() {
            return Err(DomainError::validation("customer id is required"));
        }
        if amount.amount_minor == 0 {
            return Err(DomainError::validation(
                "invoice amount must be greater than zero",
            ));
        }
        Ok(Self {
            id,
            order_id,
            customer_id,
            amount,
            status: InvoiceStatus::Draft,
            created_at: now,
            issued_at: None,
            paid_at: None,
        })
    }

    pub fn issue(&mut self, now: DateTime<Utc>) -> Result<(), DomainError> {
        if !self.status.can_issue() {
            return Err(DomainError::invalid_transition(format!(
                "cannot issue an invoice in status {}",
                self.status
            )));
        }
        self.status = InvoiceStatus::Issued;
        self.issued_at = Some(now);
        Ok(())
    }

    pub fn pay(&mut self, now: DateTime<Utc>) -> Result<(), DomainError> {
        if !self.status.can_pay() {
            return Err(DomainError::invalid_transition(format!(
                "cannot pay an invoice in status {}",
                self.status
            )));
        }
        self.status = InvoiceStatus::Paid;
        self.paid_at = Some(now);
        Ok(())
    }

    pub fn cancel(&mut self) -> Result<(), DomainError> {
        if !self.status.can_cancel() {
            return Err(DomainError::invalid_transition(format!(
                "cannot cancel an invoice in status {}",
                self.status
            )));
        }
        self.status = InvoiceStatus::Cancelled;
        Ok(())
    }

    pub fn refund(&mut self) -> Result<(), DomainError> {
        if !self.status.can_refund() {
            return Err(DomainError::invalid_transition(format!(
                "cannot refund an invoice in status {}",
                self.status
            )));
        }
        self.status = InvoiceStatus::Refunded;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// PayoutStatus
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayoutStatus {
    Pending,
    Approved,
    Paid,
}

impl PayoutStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Paid => "paid",
        }
    }
}

impl std::str::FromStr for PayoutStatus {
    type Err = DomainError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "pending" => Ok(Self::Pending),
            "approved" => Ok(Self::Approved),
            "paid" => Ok(Self::Paid),
            other => Err(DomainError::validation(format!(
                "unknown payout status: {other}"
            ))),
        }
    }
}

impl std::fmt::Display for PayoutStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

// ---------------------------------------------------------------------------
// CourierPayout entity
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CourierPayout {
    pub id: Uuid,
    pub courier_id: Uuid,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub gross_amount: Money,
    pub commission: Money,
    pub net_amount: Money,
    pub status: PayoutStatus,
    pub created_at: DateTime<Utc>,
}

impl CourierPayout {
    pub fn create(
        courier_id: Uuid,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        gross_amount: Money,
        commission: Money,
        now: DateTime<Utc>,
    ) -> Result<Self, DomainError> {
        if courier_id.is_nil() {
            return Err(DomainError::validation("courier id is required"));
        }
        if period_end <= period_start {
            return Err(DomainError::validation(
                "period end must be after period start",
            ));
        }
        if gross_amount.currency != commission.currency {
            return Err(DomainError::validation(
                "gross amount and commission must share the same currency",
            ));
        }
        let net_minor = gross_amount.amount_minor - commission.amount_minor;
        if net_minor < 0 {
            return Err(DomainError::validation(
                "commission cannot exceed gross amount",
            ));
        }
        let net_amount = Money {
            amount_minor: net_minor,
            currency: gross_amount.currency.clone(),
        };
        Ok(Self {
            id: Uuid::now_v7(),
            courier_id,
            period_start,
            period_end,
            gross_amount,
            commission,
            net_amount,
            status: PayoutStatus::Pending,
            created_at: now,
        })
    }

    pub fn approve(&mut self) -> Result<(), DomainError> {
        if self.status != PayoutStatus::Pending {
            return Err(DomainError::invalid_transition(
                "only pending payouts can be approved",
            ));
        }
        self.status = PayoutStatus::Approved;
        Ok(())
    }

    pub fn mark_paid(&mut self) -> Result<(), DomainError> {
        if self.status != PayoutStatus::Approved {
            return Err(DomainError::invalid_transition(
                "only approved payouts can be marked as paid",
            ));
        }
        self.status = PayoutStatus::Paid;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_invoice() -> Invoice {
        Invoice::create(
            InvoiceId::new(),
            OrderId::new(),
            Uuid::now_v7(),
            Money::new(5_000, "TRY").unwrap(),
            Utc::now(),
        )
        .expect("valid invoice")
    }

    // ---- Invoice tests ----

    #[test]
    fn invoice_starts_as_draft() {
        let inv = sample_invoice();
        assert_eq!(inv.status, InvoiceStatus::Draft);
        assert!(inv.issued_at.is_none());
        assert!(inv.paid_at.is_none());
    }

    #[test]
    fn invoice_full_lifecycle_draft_issue_pay_refund() {
        let mut inv = sample_invoice();
        inv.issue(Utc::now()).expect("issue");
        assert_eq!(inv.status, InvoiceStatus::Issued);

        inv.pay(Utc::now()).expect("pay");
        assert_eq!(inv.status, InvoiceStatus::Paid);
        assert!(inv.paid_at.is_some());

        inv.refund().expect("refund");
        assert_eq!(inv.status, InvoiceStatus::Refunded);
    }

    #[test]
    fn cannot_pay_a_draft_invoice() {
        let mut inv = sample_invoice();
        let err = inv.pay(Utc::now()).unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn cannot_refund_an_issued_invoice() {
        let mut inv = sample_invoice();
        inv.issue(Utc::now()).expect("issue");
        let err = inv.refund().unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }

    #[test]
    fn rejects_zero_amount_invoice() {
        let result = Invoice::create(
            InvoiceId::new(),
            OrderId::new(),
            Uuid::now_v7(),
            Money::new(0, "TRY").unwrap(),
            Utc::now(),
        );
        assert!(result.is_err());
    }

    // ---- CourierPayout tests ----

    #[test]
    fn payout_calculates_net_amount() {
        let payout = CourierPayout::create(
            Uuid::now_v7(),
            Utc::now(),
            Utc::now() + chrono::Duration::days(7),
            Money::new(10_000, "TRY").unwrap(),
            Money::new(1_500, "TRY").unwrap(),
            Utc::now(),
        )
        .expect("valid payout");
        assert_eq!(payout.net_amount.amount_minor, 8_500);
        assert_eq!(payout.status, PayoutStatus::Pending);
    }

    #[test]
    fn payout_rejects_commission_exceeding_gross() {
        let result = CourierPayout::create(
            Uuid::now_v7(),
            Utc::now(),
            Utc::now() + chrono::Duration::days(7),
            Money::new(1_000, "TRY").unwrap(),
            Money::new(2_000, "TRY").unwrap(),
            Utc::now(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn payout_lifecycle_pending_approve_paid() {
        let mut payout = CourierPayout::create(
            Uuid::now_v7(),
            Utc::now(),
            Utc::now() + chrono::Duration::days(7),
            Money::new(10_000, "TRY").unwrap(),
            Money::new(1_500, "TRY").unwrap(),
            Utc::now(),
        )
        .expect("valid payout");

        payout.approve().expect("approve");
        assert_eq!(payout.status, PayoutStatus::Approved);

        payout.mark_paid().expect("paid");
        assert_eq!(payout.status, PayoutStatus::Paid);
    }

    #[test]
    fn cannot_mark_pending_payout_as_paid() {
        let mut payout = CourierPayout::create(
            Uuid::now_v7(),
            Utc::now(),
            Utc::now() + chrono::Duration::days(7),
            Money::new(10_000, "TRY").unwrap(),
            Money::new(1_500, "TRY").unwrap(),
            Utc::now(),
        )
        .expect("valid payout");
        let err = payout.mark_paid().unwrap_err();
        assert!(matches!(err, DomainError::InvalidTransition(_)));
    }
}
