// =============================================================================
// File:           mobile/android/feature/orders/src/main/kotlin/com/qervon/features/orders/ExternalNavigation.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Launches the device's preferred turn-by-turn navigation app via a
//   `google.navigation:` Intent (Google Maps handles this scheme when
//   installed), falling back to a generic `geo:` Intent so the user can
//   pick any installed maps app — no in-app map, no Maps API key. This is
//   the Android equivalent of the iOS client's external-nav-app picker.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.orders

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import java.util.Locale

object ExternalNavigation {

    fun launch(context: Context, latitude: Double, longitude: Double, label: String?) {
        val navigationIntent = Intent(
            Intent.ACTION_VIEW,
            Uri.parse(String.format(Locale.US, "google.navigation:q=%f,%f", latitude, longitude)),
        ).apply { setPackage("com.google.android.apps.maps") }

        try {
            context.startActivity(navigationIntent)
            return
        } catch (_: ActivityNotFoundException) {
            // Google Maps isn't installed — fall through to a generic geo: Intent.
        }

        val query = Uri.encode(label?.takeIf { it.isNotBlank() } ?: String.format(Locale.US, "%f,%f", latitude, longitude))
        val geoIntent = Intent(
            Intent.ACTION_VIEW,
            Uri.parse(String.format(Locale.US, "geo:%f,%f?q=%s", latitude, longitude, query)),
        )
        try {
            context.startActivity(geoIntent)
        } catch (_: ActivityNotFoundException) {
            // No maps app installed at all — nothing further we can honestly do.
        }
    }
}
