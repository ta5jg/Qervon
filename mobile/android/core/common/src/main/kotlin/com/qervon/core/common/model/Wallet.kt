// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/model/Wallet.kt
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

package com.qervon.core.common.model

import com.qervon.core.common.InstantSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import java.time.Instant

@Serializable
enum class WalletTransactionType {
    @SerialName("delivery_earning") DELIVERY_EARNING,
    @SerialName("performance_bonus") PERFORMANCE_BONUS,
    @SerialName("tip") TIP,
    @SerialName("penalty_deduction") PENALTY_DEDUCTION,
    @SerialName("payout_withdrawal") PAYOUT_WITHDRAWAL,
    ;

    fun displayName(): String = when (this) {
        DELIVERY_EARNING -> "Teslimat Hakedişi"
        PERFORMANCE_BONUS -> "Performans Primi"
        TIP -> "Bahşiş"
        PENALTY_DEDUCTION -> "Ceza Kesintisi"
        PAYOUT_WITHDRAWAL -> "Ödeme Çekimi"
    }

    /** Whether this transaction type adds to (true) or subtracts from
     * (false) the courier's balance, purely for UI sign/color purposes. */
    val isCredit: Boolean
        get() = when (this) {
            DELIVERY_EARNING, PERFORMANCE_BONUS, TIP -> true
            PENALTY_DEDUCTION, PAYOUT_WITHDRAWAL -> false
        }
}

@Serializable
data class WalletTransaction(
    val id: String,
    val transactionType: WalletTransactionType,
    val amountMinor: Long,
    val currency: String,
    val description: String,
    @Serializable(with = InstantSerializer::class)
    val createdAt: Instant,
) {
    val money: Money get() = Money(amountMinor, currency)
}

@Serializable
data class CourierWallet(
    val courierId: String,
    val balanceMinor: Long,
    val totalEarnedMinor: Long,
    val totalBonusMinor: Long,
    val totalPenaltiesMinor: Long,
    val currency: String,
    val transactions: List<WalletTransaction>,
) {
    val balance: Money get() = Money(balanceMinor, currency)
    val totalEarned: Money get() = Money(totalEarnedMinor, currency)

    /** Sums delivery-earning/bonus/tip transactions created since [since],
     * purely client-side (the backend has no period aggregation endpoint)
     * — used for the "today / this week / this month" cards. */
    fun totalCreditedSince(since: Instant): Money {
        val sum = transactions
            .filter { it.createdAt >= since && it.transactionType.isCredit }
            .sumOf { it.amountMinor }
        return Money(sum, currency)
    }

    val deliveryCount: Int get() = transactions.count { it.transactionType == WalletTransactionType.DELIVERY_EARNING }
}
