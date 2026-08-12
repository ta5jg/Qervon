-- =============================================================================
-- File:           backend/migrations/billing/00000000000003_courier_wallets.sql
-- Project:        Qervon
-- Author:         USDTG GROUP TECHNOLOGY LLC
-- Developer:      Irfan Gedik
-- Created Date:   2026-08-12
-- Version:        0.1.0
--
-- Description:
--   Persists courier wallet balances and their append-only transaction
--   ledger (delivery earnings, performance bonuses, tips, penalties, and
--   payout withdrawals).
--
-- Specification:
--   QAS-000004, QAS-000006, QLS-000009, QES-000002, QES-000006.
--
-- License:
--   Qervon License v1.0 — see LICENSE in the repository root.
-- =============================================================================

CREATE TABLE billing.courier_wallets (
    courier_id             uuid PRIMARY KEY REFERENCES couriers.couriers (id),
    balance_minor          bigint NOT NULL DEFAULT 0,
    total_earned_minor     bigint NOT NULL DEFAULT 0,
    total_bonus_minor      bigint NOT NULL DEFAULT 0,
    total_penalties_minor  bigint NOT NULL DEFAULT 0,
    currency               char(3) NOT NULL,
    created_at             timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE billing.wallet_transactions (
    id                uuid PRIMARY KEY,
    courier_id        uuid NOT NULL REFERENCES billing.courier_wallets (courier_id),
    transaction_type  text NOT NULL,
    amount_minor      bigint NOT NULL,
    currency          char(3) NOT NULL,
    description       text NOT NULL,
    created_at        timestamptz NOT NULL,
    CONSTRAINT wallet_transactions_type_check
        CHECK (transaction_type IN (
            'delivery_earning', 'performance_bonus', 'tip',
            'penalty_deduction', 'payout_withdrawal'
        ))
);

CREATE INDEX wallet_transactions_courier_idx
    ON billing.wallet_transactions (courier_id, created_at DESC);
