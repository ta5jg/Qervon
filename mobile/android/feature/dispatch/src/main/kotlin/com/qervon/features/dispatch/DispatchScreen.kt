// =============================================================================
// File:           mobile/android/feature/dispatch/src/main/kotlin/com/qervon/features/dispatch/DispatchScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.dispatch

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.qervon.core.common.model.OrderStatus
import com.qervon.core.designsystem.QervonCard
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing
import com.qervon.core.designsystem.StatusPill

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DispatchScreen(viewModel: DispatchViewModel = hiltViewModel()) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(Unit) { viewModel.refreshCourier() }

    Scaffold(topBar = { TopAppBar(title = { Text("Kurye Paneli") }) }) { padding ->
        Column(
            modifier = Modifier.fillMaxSize().padding(padding).padding(QervonSpacing.md),
            verticalArrangement = Arrangement.spacedBy(QervonSpacing.md),
        ) {
            QervonCard {
                Row(
                    modifier = Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Column {
                        Text("Durum", style = MaterialTheme.typography.titleMedium)
                        StatusPill(
                            text = if (state.isOnline) "Çevrimiçi" else "Çevrimdışı",
                            color = if (state.isOnline) QervonColors.Success else QervonColors.OnSurfaceMuted,
                        )
                    }
                    Switch(checked = state.isOnline, onCheckedChange = { viewModel.toggleOnline() }, enabled = !state.isTogglingOnline)
                }
            }

            state.activeOrder?.let { order ->
                QervonCard {
                    Text("Aktif Görev", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(QervonSpacing.sm))
                    Text(
                        if (order.status == OrderStatus.COURIER_ASSIGNED) "Alım noktasına gidiliyor" else "Teslimata gidiliyor",
                        color = QervonColors.Primary,
                        style = MaterialTheme.typography.bodySmall,
                    )
                    Spacer(Modifier.height(6.dp))
                    Text("Alım: ${order.pickup.label ?: "Konum"}")
                    Text("Teslim: ${order.dropoff.label ?: "Konum"}")
                    Text("Ücret: ${order.fare.formatted()}", style = MaterialTheme.typography.titleMedium)
                }
            }

            state.pendingOffer?.let { offer ->
                QervonCard {
                    Text("Yeni Sipariş Teklifi", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(QervonSpacing.sm))
                    Text("Alım: ${offer.order.pickup.label ?: "Konum"}")
                    Text("Teslim: ${offer.order.dropoff.label ?: "Konum"}")
                    Text("Ücret: ${offer.order.fare.formatted()}", style = MaterialTheme.typography.titleMedium)
                    Spacer(Modifier.height(QervonSpacing.sm))
                    LinearProgressIndicator(
                        progress = { (state.secondsRemaining / 30f).coerceIn(0f, 1f) },
                        modifier = Modifier.fillMaxWidth(),
                    )
                    Text("${state.secondsRemaining} sn kaldı", style = MaterialTheme.typography.labelSmall, color = QervonColors.OnSurfaceMuted)
                    Spacer(Modifier.height(QervonSpacing.sm))
                    Row(horizontalArrangement = Arrangement.spacedBy(QervonSpacing.sm)) {
                        OutlinedButton(onClick = viewModel::rejectOffer, enabled = !state.isRespondingToOffer, modifier = Modifier.weight(1f)) {
                            Text("Reddet")
                        }
                        QervonPrimaryButton(text = "Kabul et", onClick = viewModel::acceptOffer, isLoading = state.isRespondingToOffer, modifier = Modifier.weight(1f))
                    }
                }
            }

            state.errorMessage?.let {
                Surface(color = QervonColors.Danger.copy(alpha = 0.1f)) {
                    Text(it, color = QervonColors.Danger, modifier = Modifier.padding(QervonSpacing.sm))
                }
            }
        }
    }
}
