// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/model/CustomerProfile.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `qervon_domain::{CustomerProfile, SavedAddress}` as returned by
//   `GET /v1/customer/profile` and the address book endpoints.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common.model

import com.qervon.core.common.InstantSerializer
import kotlinx.serialization.Serializable
import java.time.Instant

@Serializable
data class SavedAddress(
    val id: String,
    val label: String,
    val location: GeoLocation,
    val fullAddress: String,
    val isDefault: Boolean,
)

@Serializable
data class CustomerProfile(
    val id: String,
    val userId: String,
    val companyName: String? = null,
    val taxId: String? = null,
    val addresses: List<SavedAddress> = emptyList(),
    val loyaltyPoints: Long = 0,
    @Serializable(with = InstantSerializer::class)
    val createdAt: Instant,
)
