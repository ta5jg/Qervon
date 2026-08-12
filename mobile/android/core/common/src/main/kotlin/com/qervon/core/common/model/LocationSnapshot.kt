// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/model/LocationSnapshot.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors the API gateway's `LocationUpdateEvent`
//   (`GET /v1/orders/{id}/tracking`) — a single courier location sample.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common.model

import com.qervon.core.common.InstantSerializer
import kotlinx.serialization.Serializable
import java.time.Instant

@Serializable
data class LocationSnapshot(
    val courierId: String,
    val tenantId: String,
    val latitude: Double,
    val longitude: Double,
    @Serializable(with = InstantSerializer::class)
    val timestamp: Instant,
    val fraudFlagged: Boolean = false,
    val fraudRiskScore: Double = 0.0,
) {
    val coordinate: GeoLocation get() = GeoLocation(latitude, longitude)
}
