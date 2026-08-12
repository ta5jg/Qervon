// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/model/Rating.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `CustomerRatingResponse` (`GET /v1/courier/me/ratings`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common.model

import com.qervon.core.common.InstantSerializer
import kotlinx.serialization.Serializable
import java.time.Instant

@Serializable
data class CustomerRating(
    val id: String,
    val orderId: String,
    val customerId: String,
    val courierId: String,
    val ratingStars: Int,
    val comment: String? = null,
    val photoUrl: String? = null,
    @Serializable(with = InstantSerializer::class)
    val createdAt: Instant,
)

/** Average of all star ratings, or null when there are none yet — the UI
 * must show "no ratings yet" rather than a fabricated 0.0. */
fun List<CustomerRating>.averageStars(): Double? {
    if (isEmpty()) return null
    return sumOf { it.ratingStars }.toDouble() / size
}
