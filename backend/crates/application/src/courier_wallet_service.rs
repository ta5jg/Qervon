// =============================================================================
// File:           backend/crates/application/src/courier_wallet_service.rs
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Courier wallet use cases: read a courier's balance/ledger and credit
//   delivery earnings, bonuses, or penalties.
//
// Specification:
//   QAS-000004, QAS-000006, QLS-000009, QES-000002, QES-000006.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

use qervon_domain::{CourierWallet, CourierWalletRepository};
use uuid::Uuid;

use crate::error::ApplicationError;

pub struct CourierWalletService<R>
where
    R: CourierWalletRepository,
{
    wallets: R,
}

impl<R> CourierWalletService<R>
where
    R: CourierWalletRepository,
{
    pub fn new(wallets: R) -> Self {
        Self { wallets }
    }

    /// Reads a courier's wallet. Never creates a row: a courier with no
    /// transactions yet gets a zero-balance wallet computed in memory, so a
    /// read alone has no persistence side effects.
    pub async fn get_wallet(
        &self,
        courier_id: Uuid,
        default_currency: &str,
    ) -> Result<CourierWallet, ApplicationError> {
        Ok(self
            .wallets
            .find_by_courier(courier_id)
            .await?
            .unwrap_or_else(|| CourierWallet::new(courier_id, default_currency)))
    }

    /// Credits a completed delivery's earning to the assigned courier's
    /// wallet, creating the wallet on first use.
    pub async fn credit_delivery_earning(
        &self,
        courier_id: Uuid,
        amount_minor: i64,
        currency: &str,
        order_ref: &str,
    ) -> Result<CourierWallet, ApplicationError> {
        let mut wallet = self.get_or_create(courier_id, currency).await?;
        wallet.add_earning(amount_minor, order_ref)?;
        self.persist_last_transaction(&wallet).await?;
        Ok(wallet)
    }

    pub async fn credit_bonus(
        &self,
        courier_id: Uuid,
        amount_minor: i64,
        currency: &str,
        reason: &str,
    ) -> Result<CourierWallet, ApplicationError> {
        let mut wallet = self.get_or_create(courier_id, currency).await?;
        wallet.add_bonus(amount_minor, reason)?;
        self.persist_last_transaction(&wallet).await?;
        Ok(wallet)
    }

    pub async fn apply_penalty(
        &self,
        courier_id: Uuid,
        amount_minor: i64,
        currency: &str,
        reason: &str,
    ) -> Result<CourierWallet, ApplicationError> {
        let mut wallet = self.get_or_create(courier_id, currency).await?;
        wallet.apply_penalty(amount_minor, reason)?;
        self.persist_last_transaction(&wallet).await?;
        Ok(wallet)
    }

    async fn get_or_create(
        &self,
        courier_id: Uuid,
        currency: &str,
    ) -> Result<CourierWallet, ApplicationError> {
        if let Some(wallet) = self.wallets.find_by_courier(courier_id).await? {
            return Ok(wallet);
        }
        let wallet = CourierWallet::new(courier_id, currency);
        self.wallets.create(&wallet).await?;
        Ok(wallet)
    }

    async fn persist_last_transaction(
        &self,
        wallet: &CourierWallet,
    ) -> Result<(), ApplicationError> {
        let transaction = wallet
            .transactions
            .last()
            .expect("a mutation was just applied, so at least one transaction exists");
        self.wallets.append_transaction(wallet, transaction).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qervon_infrastructure::memory::InMemoryStore;

    #[tokio::test]
    async fn reading_an_unknown_wallet_returns_zero_balance_without_persisting() {
        let store = InMemoryStore::new();
        let service = CourierWalletService::new(store.courier_wallet_repository());
        let courier_id = Uuid::now_v7();

        let wallet = service
            .get_wallet(courier_id, "TRY")
            .await
            .expect("virtual empty wallet");
        assert_eq!(wallet.balance_minor, 0);
        assert!(store
            .courier_wallet_repository()
            .find_by_courier(courier_id)
            .await
            .expect("lookup")
            .is_none());
    }

    #[tokio::test]
    async fn crediting_earning_creates_and_persists_the_wallet() {
        let store = InMemoryStore::new();
        let service = CourierWalletService::new(store.courier_wallet_repository());
        let courier_id = Uuid::now_v7();

        let wallet = service
            .credit_delivery_earning(courier_id, 4500, "TRY", "ORD-1")
            .await
            .expect("credit earning");
        assert_eq!(wallet.balance_minor, 4500);
        assert_eq!(wallet.transactions.len(), 1);

        let reloaded = service
            .get_wallet(courier_id, "TRY")
            .await
            .expect("reload wallet");
        assert_eq!(reloaded.balance_minor, 4500);
        assert_eq!(reloaded.transactions.len(), 1);
    }

    #[tokio::test]
    async fn multiple_transactions_accumulate_in_order() {
        let store = InMemoryStore::new();
        let service = CourierWalletService::new(store.courier_wallet_repository());
        let courier_id = Uuid::now_v7();

        service
            .credit_delivery_earning(courier_id, 4500, "TRY", "ORD-1")
            .await
            .expect("earning");
        service
            .credit_bonus(courier_id, 500, "TRY", "5-star rating")
            .await
            .expect("bonus");
        let wallet = service
            .apply_penalty(courier_id, 200, "TRY", "late delivery")
            .await
            .expect("penalty");

        assert_eq!(wallet.balance_minor, 4800);
        assert_eq!(wallet.transactions.len(), 3);
    }
}
