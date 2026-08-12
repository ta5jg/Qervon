// =============================================================================
// File:           mobile/android/app-courier/src/main/kotlin/com/qervon/android/courier/ApiEnvironment.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Resolves the API gateway base URL. `10.0.2.2` is the Android
//   emulator's alias for the host machine's `localhost` — the Android
//   equivalent of the iOS simulator's `APIEnvironment` host resolution.
//   A real device on the same Wi-Fi network needs the host's LAN IP
//   instead (see mobile/android/README.md); production deployments
//   behind silhor.com require changing this constant and re-enabling TLS
//   (see the `usesCleartextTraffic` note in AndroidManifest.xml).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.android.courier

object ApiEnvironment {
    const val BASE_URL: String = "http://10.0.2.2:8080"
}
