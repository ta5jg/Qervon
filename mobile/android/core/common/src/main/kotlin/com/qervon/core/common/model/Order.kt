// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/model/Order.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `OrderResponse` (`backend/crates/api-contracts/src/lib.rs`) and
//   `qervon_domain::{OrderStatus, PaymentMethod}`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common.model

import com.qervon.core.common.InstantSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import java.time.Instant

// Note: kotlinx.serialization's JsonNamingStrategy only rewrites property
// names, not enum constant names, so every backend-facing enum here needs
// an explicit @SerialName matching the exact lowercase/snake_case string
// qervon_domain's Display/FromStr impls use.
@Serializable
enum class OrderStatus {
    @SerialName("pending") PENDING,
    @SerialName("courier_assigned") COURIER_ASSIGNED,
    @SerialName("in_transit") IN_TRANSIT,
    @SerialName("delivered") DELIVERED,
    @SerialName("cancelled") CANCELLED,
    @SerialName("returned") RETURNED,
    ;

    fun displayName(): String = when (this) {
        PENDING -> "Bekliyor"
        COURIER_ASSIGNED -> "Kurye Atandı"
        IN_TRANSIT -> "Yolda"
        DELIVERED -> "Teslim Edildi"
        CANCELLED -> "İptal"
        RETURNED -> "İade"
    }
}

@Serializable
enum class PaymentMethod {
    @SerialName("cash") CASH,
    @SerialName("card") CARD,
    @SerialName("qr") QR,
    @SerialName("wallet") WALLET,
    ;

    fun displayName(): String = when (this) {
        CASH -> "Nakit"
        CARD -> "Kart"
        QR -> "QR"
        WALLET -> "Cüzdan"
    }
}

@Serializable
data class Order(
    val id: String,
    val customerId: String,
    val pickup: Address,
    val dropoff: Address,
    val status: OrderStatus,
    val fare: Money,
    val assignedCourierId: String? = null,
    @Serializable(with = InstantSerializer::class)
    val createdAt: Instant,
    @Serializable(with = InstantSerializer::class)
    val deliveredAt: Instant? = null,
    @Serializable(with = InstantSerializer::class)
    val returnedAt: Instant? = null,
    val paymentMethod: PaymentMethod? = null,
    val paymentCollected: Boolean = false,
    val deliveryNote: String? = null,
    val contactPhone: String? = null,
)
