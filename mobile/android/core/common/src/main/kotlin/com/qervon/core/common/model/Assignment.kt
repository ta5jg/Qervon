// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/model/Assignment.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `PendingOfferResponse` (`GET /v1/courier/me/offer`) — a job
//   offered to the signed-in courier that has not yet been accepted or
//   rejected.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common.model

import com.qervon.core.common.InstantSerializer
import kotlinx.serialization.Serializable
import java.time.Instant
import kotlin.math.max

@Serializable
data class PendingOffer(
    val assignmentId: String,
    val order: Order,
    @Serializable(with = InstantSerializer::class)
    val offeredAt: Instant,
    @Serializable(with = InstantSerializer::class)
    val expiresAt: Instant,
) {
    /** Never negative — callers should treat <= 0 as "already gone". */
    fun secondsRemaining(now: Instant = Instant.now()): Long =
        max(0L, expiresAt.epochSecond - now.epochSecond)
}
