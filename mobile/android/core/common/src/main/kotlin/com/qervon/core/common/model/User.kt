// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/model/User.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `UserResponse` (returned by `POST /v1/auth/phone`, among others).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common.model

import com.qervon.core.common.InstantSerializer
import kotlinx.serialization.Serializable
import java.time.Instant

@Serializable
data class QervonUser(
    val id: String,
    val email: String,
    val displayName: String,
    val role: String,
    val status: String,
    @Serializable(with = InstantSerializer::class)
    val createdAt: Instant,
)
