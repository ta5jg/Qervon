// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/JsonConfig.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Shared kotlinx.serialization Json instance matching the backend's
//   serde_json output (snake_case field names via explicit @SerialName on
//   each model, lenient about unknown fields for forward-compatibility).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common

import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonNamingStrategy

object JsonConfig {
    /// Maps camelCase Kotlin property names to the backend's snake_case
    /// JSON field names automatically — equivalent to the iOS client's
    /// `.convertFromSnakeCase`/`.convertToSnakeCase` coder strategies, so
    /// individual models never need per-field `@SerialName` annotations.
    @OptIn(ExperimentalSerializationApi::class)
    val shared: Json = Json {
        ignoreUnknownKeys = true
        isLenient = true
        explicitNulls = false
        encodeDefaults = true
        namingStrategy = JsonNamingStrategy.SnakeCase
    }
}
