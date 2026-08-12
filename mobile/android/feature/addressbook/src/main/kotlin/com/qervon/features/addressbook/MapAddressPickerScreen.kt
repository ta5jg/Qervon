// =============================================================================
// File:           mobile/android/feature/addressbook/src/main/kotlin/com/qervon/features/addressbook/MapAddressPickerScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Tap the osmdroid map to drop a pin, reverse-geocoded to a readable
//   address via the device's built-in `android.location.Geocoder` (no
//   network geocoding API key needed) — the Android equivalent of the
//   iOS client's `MKLocalSearch`-based `MapAddressPickerView`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.addressbook

import android.location.Geocoder
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import org.osmdroid.util.GeoPoint
import java.util.Locale

data class PickedLocation(val latitude: Double, val longitude: Double, val address: String)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MapAddressPickerScreen(
    initialCenter: GeoPoint = GeoPoint(41.0082, 28.9784), // Istanbul — a sensible default center, not a real user location.
    onPicked: (PickedLocation) -> Unit,
    onClose: () -> Unit,
) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    var pickedPoint by remember { mutableStateOf<GeoPoint?>(null) }
    var address by remember { mutableStateOf("Konum seçmek için haritaya dokunun") }
    var isResolving by remember { mutableStateOf(false) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Konum Seç") },
                navigationIcon = { IconButton(onClick = onClose) { Icon(Icons.Filled.Close, contentDescription = "Kapat") } },
            )
        },
    ) { padding ->
        Column(modifier = Modifier.fillMaxSize().padding(padding)) {
            Box(modifier = Modifier.fillMaxSize().weight(1f)) {
                OsmMapView(
                    center = pickedPoint ?: initialCenter,
                    markers = pickedPoint?.let { listOf(it) } ?: emptyList(),
                    onMapTap = { point ->
                        pickedPoint = point
                        isResolving = true
                        scope.launch {
                            address = reverseGeocode(context, point.latitude, point.longitude)
                            isResolving = false
                        }
                    },
                )
            }
            Surface {
                Column(modifier = Modifier.fillMaxWidth().padding(QervonSpacing.md)) {
                    Text(
                        if (isResolving) "Adres çözümleniyor…" else address,
                        style = MaterialTheme.typography.bodyMedium,
                    )
                    androidx.compose.foundation.layout.Spacer(Modifier.padding(QervonSpacing.xs))
                    QervonPrimaryButton(
                        text = "Bu Konumu Kullan",
                        enabled = pickedPoint != null && !isResolving,
                        onClick = {
                            pickedPoint?.let { onPicked(PickedLocation(it.latitude, it.longitude, address)) }
                        },
                    )
                }
            }
        }
    }
}

private suspend fun reverseGeocode(context: android.content.Context, latitude: Double, longitude: Double): String =
    withContext(Dispatchers.IO) {
        try {
            @Suppress("DEPRECATION") // The async Geocoder overload requires API 33; this stays compatible with minSdk 26.
            val results = Geocoder(context, Locale.getDefault()).getFromLocation(latitude, longitude, 1)
            results?.firstOrNull()?.let { address ->
                listOfNotNull(address.thoroughfare, address.subLocality, address.locality)
                    .joinToString(", ")
                    .ifBlank { String.format(Locale.US, "%.5f, %.5f", latitude, longitude) }
            } ?: String.format(Locale.US, "%.5f, %.5f", latitude, longitude)
        } catch (_: Exception) {
            String.format(Locale.US, "%.5f, %.5f", latitude, longitude)
        }
    }
