package com.qervon.features.customerorder

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.qervon.core.common.model.Order
import com.qervon.core.common.model.OrderStatus
import com.qervon.core.designsystem.QervonCard
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing
import com.qervon.features.addressbook.OsmMapView
import org.osmdroid.util.GeoPoint

@Composable
fun CustomerLiveTrackingScreen(
    viewModel: OrderHistoryViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val trackedOrder = state.orders.firstOrNull { it.status == OrderStatus.COURIER_ASSIGNED || it.status == OrderStatus.IN_TRANSIT }
    val mapCenter = trackedOrder?.pickup?.let { GeoPoint(it.latitude, it.longitude) } ?: GeoPoint(41.0082, 28.9784)
    val markers = trackedOrder?.let { listOf(GeoPoint(it.pickup.latitude, it.pickup.longitude), GeoPoint(it.dropoff.latitude, it.dropoff.longitude)) } ?: emptyList()

    LaunchedEffect(Unit) {
        viewModel.refresh()
        viewModel.startLiveUpdates()
    }
    DisposableEffect(Unit) {
        onDispose { viewModel.stopLiveUpdates() }
    }

    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(QervonSpacing.md),
        verticalArrangement = Arrangement.spacedBy(QervonSpacing.md),
    ) {
        item {
            QervonCard {
                Column(verticalArrangement = Arrangement.spacedBy(QervonSpacing.sm)) {
                    OsmMapView(
                        modifier = Modifier
                            .fillMaxWidth()
                            .height(220.dp),
                        center = mapCenter,
                        markers = markers,
                    )
                    Text(
                        "ATANAN SÜRÜCÜ",
                        style = MaterialTheme.typography.labelSmall,
                        color = QervonColors.OnSurfaceMuted,
                    )
                    Text(
                        if (trackedOrder == null) "Aktif sipariş bekleniyor" else "Atanan kurye canlı takipte",
                        color = QervonColors.Primary,
                        style = MaterialTheme.typography.titleMedium,
                    )
                    Text(
                        if (trackedOrder == null) "Kurye ataması bekleniyor" else "TAHMİNİ ETA: 3 Dk",
                        style = MaterialTheme.typography.bodySmall,
                        color = QervonColors.OnSurfaceMuted,
                    )
                    QervonPrimaryButton(
                        text = "BİLDİRİMLERİ AÇ",
                        onClick = {},
                    )
                }
            }
        }
        item {
            QervonCard {
                Text("Hızlı Kurye Çağır", style = MaterialTheme.typography.titleMedium)
                Text(
                    "Yeni siparişleri Sipariş Ver sekmesinden oluşturabilirsiniz.",
                    style = MaterialTheme.typography.bodySmall,
                    color = QervonColors.OnSurfaceMuted,
                    modifier = Modifier.padding(top = QervonSpacing.xs),
                )
            }
        }
        items(state.orders.take(3), key = { it.id }) { order ->
            TrackedOrderSummary(order = order)
        }
    }
}

@Composable
private fun TrackedOrderSummary(order: Order) {
    QervonCard {
        Text(order.status.displayName(), color = QervonColors.Secondary, style = MaterialTheme.typography.labelSmall)
        Text(
            "${order.pickup.label ?: "Alım"} ➔ ${order.dropoff.label ?: "Teslim"}",
            style = MaterialTheme.typography.bodyMedium,
            modifier = Modifier.padding(top = QervonSpacing.xs),
        )
        Text(
            "Ücret: ${order.fare.formatted()}",
            style = MaterialTheme.typography.bodySmall,
            color = QervonColors.OnSurfaceMuted,
            modifier = Modifier.padding(top = QervonSpacing.xs),
        )
    }
}
