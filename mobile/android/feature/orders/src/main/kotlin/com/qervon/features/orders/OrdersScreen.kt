// =============================================================================
// File:           mobile/android/feature/orders/src/main/kotlin/com/qervon/features/orders/OrdersScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.orders

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.qervon.core.common.model.Order
import com.qervon.core.common.model.OrderStatus
import com.qervon.core.designsystem.QervonCard
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing
import com.qervon.core.designsystem.StatusPill

@Composable
fun OrdersScreen(
    onStartPickup: (orderId: String) -> Unit,
    onStartDelivery: (orderId: String, isCashOrder: Boolean) -> Unit,
    viewModel: OrdersViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    val context = LocalContext.current

    LaunchedEffect(Unit) {
        viewModel.refresh()
        viewModel.startLiveUpdates()
    }
    DisposableEffect(Unit) {
        onDispose { viewModel.stopLiveUpdates() }
    }

    if (state.orders.isEmpty() && !state.isLoading) {
        Column(modifier = Modifier.fillMaxSize().padding(QervonSpacing.lg)) {
            Text("Şu anda atanmış aktif iş yok.", color = QervonColors.OnSurfaceMuted)
        }
        return
    }
    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(QervonSpacing.md),
        verticalArrangement = Arrangement.spacedBy(QervonSpacing.md),
    ) {
        items(state.orders, key = { it.id }) { order ->
            OrderCard(
                order = order,
                isProcessing = state.processingOrderId == order.id,
                onNavigatePickup = { ExternalNavigation.launch(context, order.pickup.latitude, order.pickup.longitude, order.pickup.label) },
                onNavigateDropoff = { ExternalNavigation.launch(context, order.dropoff.latitude, order.dropoff.longitude, order.dropoff.label) },
                onPickup = { onStartPickup(order.id) },
                onDeliver = { onStartDelivery(order.id, order.paymentMethod == com.qervon.core.common.model.PaymentMethod.CASH) },
            )
        }
    }
}

@Composable
private fun OrderCard(
    order: Order,
    isProcessing: Boolean,
    onNavigatePickup: () -> Unit,
    onNavigateDropoff: () -> Unit,
    onPickup: () -> Unit,
    onDeliver: () -> Unit,
) {
    QervonCard {
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Text(order.fare.formatted(), style = MaterialTheme.typography.titleMedium)
            StatusPill(
                text = order.status.displayName(),
                color = if (order.status == OrderStatus.IN_TRANSIT) QervonColors.Primary else QervonColors.Warning,
            )
        }
        Spacer(Modifier.height(QervonSpacing.sm))
        Text("Alım: ${order.pickup.label ?: "Konum belirtilmedi"}")
        Text("Teslim: ${order.dropoff.label ?: "Konum belirtilmedi"}")
        order.deliveryNote?.takeIf { it.isNotBlank() }?.let {
            Text("Not: $it", style = MaterialTheme.typography.bodyMedium, color = QervonColors.OnSurfaceMuted)
        }
        Spacer(Modifier.height(QervonSpacing.sm))
        if (order.status == OrderStatus.COURIER_ASSIGNED) {
            Row(horizontalArrangement = Arrangement.spacedBy(QervonSpacing.sm)) {
                OutlinedButton(onClick = onNavigatePickup) { Text("Alım için Yol Tarifi") }
            }
            Spacer(Modifier.height(QervonSpacing.sm))
            QervonPrimaryButton(text = "Teslim Aldım", onClick = onPickup, isLoading = isProcessing)
        } else {
            Row(horizontalArrangement = Arrangement.spacedBy(QervonSpacing.sm)) {
                OutlinedButton(onClick = onNavigateDropoff) { Text("Teslimat için Yol Tarifi") }
            }
            Spacer(Modifier.height(QervonSpacing.sm))
            QervonPrimaryButton(text = "Teslim Et", onClick = onDeliver)
        }
    }
}
