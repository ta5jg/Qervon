// =============================================================================
// File:           mobile/android/app-customer/src/main/kotlin/com/qervon/android/customer/ApiEnvironment.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   See app-courier's `ApiEnvironment.kt` for the full rationale — same
//   emulator-host resolution, duplicated per-app since each app is a
//   fully independent Gradle module (no shared "app config" module).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.android.customer

object ApiEnvironment {
    const val BASE_URL: String = "http://10.0.2.2:8080"
}
