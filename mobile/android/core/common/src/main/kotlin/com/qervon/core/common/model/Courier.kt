// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/model/Courier.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `CourierResponse` (`backend/crates/api-contracts/src/lib.rs`).
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
enum class VehicleType {
    @SerialName("bicycle") BICYCLE,
    @SerialName("motorcycle") MOTORCYCLE,
    @SerialName("car") CAR,
    ;

    fun displayName(): String = when (this) {
        BICYCLE -> "Bisiklet"
        MOTORCYCLE -> "Motosiklet"
        CAR -> "Otomobil"
    }
}

@Serializable
enum class CourierStatus {
    @SerialName("available") AVAILABLE,
    @SerialName("busy") BUSY,
    @SerialName("offline") OFFLINE,
}

@Serializable
data class Courier(
    val id: String,
    val name: String,
    val vehicle: VehicleType,
    val status: CourierStatus,
    val currentLocation: GeoLocation? = null,
    @Serializable(with = InstantSerializer::class)
    val registeredAt: Instant,
)
