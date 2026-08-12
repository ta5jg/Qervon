// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/model/FareQuote.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `FareQuoteResponse` (`GET /v1/customer/fare-quote`) and
//   `EtaResponse` (`GET /v1/customer/orders/{id}/eta`). Both are
//   non-binding estimates — the server always recomputes the authoritative
//   fare at order-creation time, and the ETA has no real traffic data.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common.model

import kotlinx.serialization.Serializable

@Serializable
data class FareQuote(
    val fareAmountMinor: Long,
    val currency: String,
    val distanceKm: Double,
) {
    val money: Money get() = Money(fareAmountMinor, currency)
}

@Serializable
data class EtaInfo(
    val etaMinutes: Double,
    val distanceKm: Double,
)
