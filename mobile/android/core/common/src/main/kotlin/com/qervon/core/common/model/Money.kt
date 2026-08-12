// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/model/Money.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `MoneyDto` (`{amount_minor, currency}`). Amounts are always
//   minor units (kuruş), matching the backend's `Money` value object.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common.model

import kotlinx.serialization.Serializable
import java.text.NumberFormat
import java.util.Currency
import java.util.Locale

@Serializable
data class Money(
    val amountMinor: Long,
    val currency: String,
) {
    /** Formats as a localized currency string, e.g. "₺45,00". */
    fun formatted(): String {
        val value = amountMinor / 100.0
        return try {
            val formatter = NumberFormat.getCurrencyInstance(Locale("tr", "TR"))
            formatter.currency = Currency.getInstance(currency)
            formatter.format(value)
        } catch (_: IllegalArgumentException) {
            "$value $currency"
        }
    }
}
