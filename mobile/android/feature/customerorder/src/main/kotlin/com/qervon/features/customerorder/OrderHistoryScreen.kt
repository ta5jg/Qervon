// =============================================================================
// File:           mobile/android/feature/customerorder/src/main/kotlin/com/qervon/features/customerorder/OrderHistoryScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.customerorder

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
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
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.qervon.core.common.QervonFormat
import com.qervon.core.common.model.Order
import com.qervon.core.common.model.OrderStatus
import com.qervon.core.designsystem.QervonCard
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonSpacing
import com.qervon.core.designsystem.StatusPill

@Composable
fun OrderHistoryScreen(
    onOrderSelected: (orderId: String) -> Unit,
    viewModel: OrderHistoryViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(Unit) {
        viewModel.refresh()
        viewModel.startLiveUpdates()
    }
    DisposableEffect(Unit) {
        onDispose { viewModel.stopLiveUpdates() }
    }

    if (state.orders.isEmpty() && !state.isLoading) {
        Column(modifier = Modifier.fillMaxSize().padding(QervonSpacing.lg)) {
            Text("Henüz siparişiniz yok.", color = QervonColors.OnSurfaceMuted)
        }
        return
    }
    LazyColumn(
        modifier = Modifier.fillMaxSize(),
        contentPadding = PaddingValues(QervonSpacing.md),
        verticalArrangement = Arrangement.spacedBy(QervonSpacing.sm),
    ) {
        items(state.orders, key = { it.id }) { order -> OrderRow(order, onClick = { onOrderSelected(order.id) }) }
    }
}

@Composable
private fun OrderRow(order: Order, onClick: () -> Unit) {
    QervonCard(modifier = Modifier.fillMaxWidth().clickable(onClick = onClick)) {
        Row(
            modifier = Modifier.fillMaxWidth(),
            horizontalArrangement = Arrangement.SpaceBetween,
        ) {
            Column {
                Text(order.dropoff.label ?: "Teslim konumu", style = MaterialTheme.typography.bodyMedium)
                Text(QervonFormat.dayAndTime(order.createdAt), color = QervonColors.OnSurfaceMuted, style = MaterialTheme.typography.labelSmall)
            }
            Column(horizontalAlignment = androidx.compose.ui.Alignment.End) {
                Text(order.fare.formatted(), style = MaterialTheme.typography.titleMedium)
                StatusPill(
                    text = order.status.displayName(),
                    color = when (order.status) {
                        OrderStatus.DELIVERED -> QervonColors.Success
                        OrderStatus.CANCELLED, OrderStatus.RETURNED -> QervonColors.Danger
                        else -> QervonColors.Primary
                    },
                )
            }
        }
    }
}
