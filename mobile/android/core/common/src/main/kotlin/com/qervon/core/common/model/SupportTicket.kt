// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/model/SupportTicket.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `SupportTicketResponse` (`POST/GET /v1/customer/support-tickets`).
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
enum class TicketStatus {
    @SerialName("open") OPEN,
    @SerialName("in_progress") IN_PROGRESS,
    @SerialName("resolved") RESOLVED,
    @SerialName("closed") CLOSED,
    ;

    fun displayName(): String = when (this) {
        OPEN -> "Açık"
        IN_PROGRESS -> "İşlemde"
        RESOLVED -> "Çözüldü"
        CLOSED -> "Kapatıldı"
    }
}

@Serializable
data class SupportTicket(
    val id: String,
    val customerId: String,
    val orderId: String? = null,
    val subject: String,
    val message: String,
    val status: TicketStatus,
    @Serializable(with = InstantSerializer::class)
    val createdAt: Instant,
)
