// =============================================================================
// File:           mobile/android/core/location/src/main/kotlin/com/qervon/core/location/LocationReporter.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Deliberately narrow seam between this module and `core:network`: the
//   Service posts samples through this interface rather than depending on
//   `QervonApi` directly, so `core:location` stays independent of the
//   networking stack and the app module wires the real implementation in.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.location

interface LocationReporter {
    /**
     * Sends one GPS sample to the backend. Returns `true` on success and
     * `false` on any failure (network drop, timeout, non-2xx response) so
     * the caller (see [CourierLocationService]'s retry queue) knows whether
     * to keep the sample for a retry on the next location tick, rather than
     * silently dropping it.
     */
    suspend fun reportLocation(latitude: Double, longitude: Double, speedKmh: Double?, batteryPct: Int?): Boolean
}
