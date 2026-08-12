// =============================================================================
// File:           mobile/android/feature/addressbook/src/main/kotlin/com/qervon/features/addressbook/OsmMapView.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Thin Compose wrapper around osmdroid's `MapView` — a real,
//   OpenStreetMap-tiled map with no API key or billing account, unlike
//   the Google Maps SDK. Reused by both the address picker here and the
//   live order-tracking screen in `feature:customerorder`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.addressbook

import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import org.osmdroid.config.Configuration
import org.osmdroid.events.MapEventsReceiver
import org.osmdroid.tileprovider.tilesource.TileSourceFactory
import org.osmdroid.util.GeoPoint
import org.osmdroid.views.MapView
import org.osmdroid.views.overlay.Marker
import org.osmdroid.views.overlay.MapEventsOverlay

/** Initializes osmdroid's global config once per process — required
 * before any [MapView] is created (tile cache dir + a compliant
 * User-Agent, since the public OSM tile servers reject requests without
 * one). */
fun configureOsmdroid(context: android.content.Context) {
    val config = Configuration.getInstance()
    config.userAgentValue = context.packageName
    config.osmdroidTileCache = context.cacheDir
}

@Composable
fun OsmMapView(
    modifier: Modifier = Modifier,
    center: GeoPoint,
    zoom: Double = 15.0,
    markers: List<GeoPoint> = emptyList(),
    onMapTap: ((GeoPoint) -> Unit)? = null,
) {
    val context = LocalContext.current
    DisposableEffect(Unit) { configureOsmdroid(context); onDispose { } }

    AndroidView(
        modifier = modifier.fillMaxSize(),
        factory = { ctx ->
            MapView(ctx).apply {
                setTileSource(TileSourceFactory.MAPNIK)
                setMultiTouchControls(true)
                controller.setZoom(zoom)
                controller.setCenter(center)
            }
        },
        update = { mapView ->
            mapView.overlays.clear()
            if (onMapTap != null) {
                val receiver = object : MapEventsReceiver {
                    override fun singleTapConfirmedHelper(point: GeoPoint): Boolean {
                        onMapTap(point)
                        return true
                    }

                    override fun longPressHelper(point: GeoPoint): Boolean = false
                }
                mapView.overlays.add(MapEventsOverlay(receiver))
            }
            markers.forEach { point ->
                mapView.overlays.add(Marker(mapView).apply { position = point })
            }
            mapView.invalidate()
        },
    )
}
