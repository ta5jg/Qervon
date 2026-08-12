// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/AuthTokenStore.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Abstraction the network layer depends on for reading/writing tokens,
//   without depending on the concrete EncryptedSharedPreferences-backed
//   implementation in `core:security` (which requires an Android Context).
//   Keeps `core:network` a pure-JVM module.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common

import com.qervon.core.common.model.AuthTokens

interface AuthTokenStore {
    fun currentTokens(): AuthTokens?
    fun save(tokens: AuthTokens)
    fun clear()
}
