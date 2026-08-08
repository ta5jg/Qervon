// =============================================================================
// File:           backend/crates/domain/src/courier_wallet.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-08
// Version:        0.1.0
//
// Description:
//   Courier Wallet domain model handling delivery earnings, performance bonuses,
//   penalty deductions, and net balance settlements.
//
// Specification:
//   QAS-000004, QES-000006.
// =============================================================================

use crate::{DomainError, Money};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalletTransactionType {
    DeliveryEarning,
    PerformanceBonus,
    Tip,
    PenaltyDeduction,
    PayoutWithdrawal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletTransaction {
    pub id: uuid::Uuid,
    pub transaction_type: WalletTransactionType,
    pub amount_minor: i64,
    pub currency: String,
    pub description: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourierWallet {
    pub courier_id: uuid::Uuid,
    pub balance_minor: i64,
    pub total_earned_minor: i64,
    pub total_bonus_minor: i64,
    pub total_penalties_minor: i64,
    pub currency: String,
    pub transactions: Vec<WalletTransaction>,
}

impl CourierWallet {
    pub fn new(courier_id: uuid::Uuid, currency: impl Into<String>) -> Self {
        Self {
            courier_id,
            balance_minor: 0,
            total_earned_minor: 0,
            total_bonus_minor: 0,
            total_penalties_minor: 0,
            currency: currency.into(),
            transactions: Vec::new(),
        }
    }

    /// Add delivery earning to courier wallet
    pub fn add_earning(&mut self, amount_minor: i64, order_ref: &str) -> Result<(), DomainError> {
        if amount_minor <= 0 {
            return Err(DomainError::Validation("Earning amount must be positive".into()));
        }

        self.balance_minor += amount_minor;
        self.total_earned_minor += amount_minor;

        self.transactions.push(WalletTransaction {
            id: uuid::Uuid::now_v7(),
            transaction_type: WalletTransactionType::DeliveryEarning,
            amount_minor,
            currency: self.currency.clone(),
            description: format!("Teslimat Hakedişi: Order #{}", order_ref),
            created_at: chrono::Utc::now(),
        });

        Ok(())
    }

    /// Add performance bonus (e.g. 100+ delivery milestone or 5-star rating bonus)
    pub fn add_bonus(&mut self, amount_minor: i64, bonus_reason: &str) -> Result<(), DomainError> {
        if amount_minor <= 0 {
            return Err(DomainError::Validation("Bonus amount must be positive".into()));
        }

        self.balance_minor += amount_minor;
        self.total_bonus_minor += amount_minor;

        self.transactions.push(WalletTransaction {
            id: uuid::Uuid::now_v7(),
            transaction_type: WalletTransactionType::PerformanceBonus,
            amount_minor,
            currency: self.currency.clone(),
            description: format!("Performans Primi: {}", bonus_reason),
            created_at: chrono::Utc::now(),
        });

        Ok(())
    }

    /// Apply penalty deduction (e.g. late delivery penalty or package damage)
    pub fn apply_penalty(&mut self, amount_minor: i64, penalty_reason: &str) -> Result<(), DomainError> {
        if amount_minor <= 0 {
            return Err(DomainError::Validation("Penalty amount must be positive".into()));
        }

        self.balance_minor -= amount_minor;
        self.total_penalties_minor += amount_minor;

        self.transactions.push(WalletTransaction {
            id: uuid::Uuid::now_v7(),
            transaction_type: WalletTransactionType::PenaltyDeduction,
            amount_minor,
            currency: self.currency.clone(),
            description: format!("Kesinti / Ceza: {}", penalty_reason),
            created_at: chrono::Utc::now(),
        });

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wallet_lifecycle_earnings_bonus_penalties() {
        let courier_id = uuid::Uuid::now_v7();
        let mut wallet = CourierWallet::new(courier_id, "TRY");

        wallet.add_earning(4500, "ORD-101").unwrap(); // ₺45.00
        wallet.add_bonus(500, "5-Star Rating").unwrap(); // ₺5.00
        wallet.apply_penalty(200, "Late Delivery").unwrap(); // -₺2.00

        assert_eq!(wallet.balance_minor, 4800); // ₺48.00 net balance
        assert_eq!(wallet.total_earned_minor, 4500);
        assert_eq!(wallet.total_bonus_minor, 500);
        assert_eq!(wallet.total_penalties_minor, 200);
        assert_eq!(wallet.transactions.len(), 3);
    }
}
