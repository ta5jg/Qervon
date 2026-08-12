// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/Wallet.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `CourierWalletResponse` / `WalletTransactionResponse`
//   (`GET /v1/courier/me/wallet`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public enum WalletTransactionType: String, Codable, Sendable, Equatable {
    case deliveryEarning = "delivery_earning"
    case performanceBonus = "performance_bonus"
    case tip
    case penaltyDeduction = "penalty_deduction"
    case payoutWithdrawal = "payout_withdrawal"

    public var displayName: String {
        switch self {
        case .deliveryEarning: return "Teslimat Hakedişi"
        case .performanceBonus: return "Performans Primi"
        case .tip: return "Bahşiş"
        case .penaltyDeduction: return "Ceza Kesintisi"
        case .payoutWithdrawal: return "Ödeme Çekimi"
        }
    }

    /// Whether this transaction type adds to (`true`) or subtracts from
    /// (`false`) the courier's balance, purely for UI sign/color purposes.
    public var isCredit: Bool {
        switch self {
        case .deliveryEarning, .performanceBonus, .tip: return true
        case .penaltyDeduction, .payoutWithdrawal: return false
        }
    }
}

public struct WalletTransaction: Codable, Sendable, Equatable, Identifiable {
    public let id: UUID
    public let transactionType: WalletTransactionType
    public let amountMinor: Int64
    public let currency: String
    public let description: String
    public let createdAt: Date

    public init(
        id: UUID,
        transactionType: WalletTransactionType,
        amountMinor: Int64,
        currency: String,
        description: String,
        createdAt: Date
    ) {
        self.id = id
        self.transactionType = transactionType
        self.amountMinor = amountMinor
        self.currency = currency
        self.description = description
        self.createdAt = createdAt
    }

    public var money: Money { Money(amountMinor: amountMinor, currency: currency) }
}

public struct CourierWallet: Codable, Sendable, Equatable {
    public let courierId: UUID
    public let balanceMinor: Int64
    public let totalEarnedMinor: Int64
    public let totalBonusMinor: Int64
    public let totalPenaltiesMinor: Int64
    public let currency: String
    public let transactions: [WalletTransaction]

    public init(
        courierId: UUID,
        balanceMinor: Int64,
        totalEarnedMinor: Int64,
        totalBonusMinor: Int64,
        totalPenaltiesMinor: Int64,
        currency: String,
        transactions: [WalletTransaction]
    ) {
        self.courierId = courierId
        self.balanceMinor = balanceMinor
        self.totalEarnedMinor = totalEarnedMinor
        self.totalBonusMinor = totalBonusMinor
        self.totalPenaltiesMinor = totalPenaltiesMinor
        self.currency = currency
        self.transactions = transactions
    }

    public var balance: Money { Money(amountMinor: balanceMinor, currency: currency) }
    public var totalEarned: Money { Money(amountMinor: totalEarnedMinor, currency: currency) }

    /// Sums delivery-earning-and-bonus-and-tip transactions created since
    /// `since`, purely client-side (the backend has no period aggregation
    /// endpoint) — used for the "today / this week / this month" cards.
    public func totalCredited(since: Date) -> Money {
        let sum = transactions
            .filter { $0.createdAt >= since && $0.transactionType.isCredit }
            .reduce(Int64(0)) { $0 + $1.amountMinor }
        return Money(amountMinor: sum, currency: currency)
    }

    public var deliveryCount: Int {
        transactions.filter { $0.transactionType == .deliveryEarning }.count
    }
}
