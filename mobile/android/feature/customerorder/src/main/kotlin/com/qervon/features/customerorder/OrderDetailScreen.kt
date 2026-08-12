// =============================================================================
// File:           mobile/android/feature/customerorder/src/main/kotlin/com/qervon/features/customerorder/OrderDetailScreen.kt
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

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.qervon.core.designsystem.QervonCard
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing
import com.qervon.core.designsystem.StatusPill
import com.qervon.features.addressbook.OsmMapView
import org.osmdroid.util.GeoPoint

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun OrderDetailScreen(
    onBack: () -> Unit,
    viewModel: OrderDetailViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    var showRatingDialog by remember { mutableStateOf(false) }
    var showSupportDialog by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) { viewModel.start() }

    Scaffold(topBar = { TopAppBar(title = { Text("Sipariş Detayı") }) }) { padding ->
        val order = state.order
        if (order == null) {
            Column(modifier = Modifier.fillMaxSize().padding(padding).padding(QervonSpacing.lg)) {
                Text(if (state.isLoading) "Yükleniyor…" else "Sipariş bulunamadı.")
            }
            return@Scaffold
        }

        Column(modifier = Modifier.fillMaxSize().padding(padding).verticalScroll(rememberScrollState())) {
            if (state.isTrackable) {
                Box(modifier = Modifier.fillMaxWidth().height(240.dp)) {
                    val courierPoint = state.courierLocation?.let { GeoPoint(it.latitude, it.longitude) }
                    OsmMapView(
                        center = courierPoint ?: GeoPoint(order.pickup.latitude, order.pickup.longitude),
                        markers = listOfNotNull(
                            courierPoint,
                            GeoPoint(order.pickup.latitude, order.pickup.longitude),
                            GeoPoint(order.dropoff.latitude, order.dropoff.longitude),
                        ),
                    )
                }
            }

            Column(modifier = Modifier.padding(QervonSpacing.md), verticalArrangement = Arrangement.spacedBy(QervonSpacing.md)) {
                QervonCard {
                    Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                        Text(order.fare.formatted(), style = MaterialTheme.typography.headlineMedium)
                        StatusPill(text = order.status.displayName(), color = QervonColors.Primary)
                    }
                    state.eta?.let { Text("Tahmini süre: ${it.etaMinutes.toInt()} dk", color = QervonColors.OnSurfaceMuted) }
                    Text("Alım: ${order.pickup.label ?: "Konum"}")
                    Text("Teslim: ${order.dropoff.label ?: "Konum"}")
                }

                state.infoMessage?.let { Text(it, color = QervonColors.Success) }
                state.errorMessage?.let { Text(it, color = QervonColors.Danger) }

                if (state.canCancel) {
                    OutlinedButton(onClick = viewModel::cancelOrder, modifier = Modifier.fillMaxWidth()) { Text("Siparişi İptal Et") }
                }
                if (state.canRate) {
                    QervonPrimaryButton(text = "Teslimatı Değerlendir", onClick = { showRatingDialog = true })
                }
                OutlinedButton(onClick = { showSupportDialog = true }, modifier = Modifier.fillMaxWidth()) { Text("Destek Talebi Oluştur") }
            }
        }
    }

    if (showRatingDialog) {
        RatingDialog(
            onDismiss = { showRatingDialog = false },
            onSubmit = { stars, comment -> viewModel.submitRating(stars, comment); showRatingDialog = false },
        )
    }
    if (showSupportDialog) {
        SupportTicketDialog(
            onDismiss = { showSupportDialog = false },
            onSubmit = { subject, message -> viewModel.submitSupportTicket(subject, message); showSupportDialog = false },
        )
    }
}

@Composable
private fun RatingDialog(onDismiss: () -> Unit, onSubmit: (stars: Int, comment: String?) -> Unit) {
    var stars by remember { mutableIntStateOf(5) }
    var comment by remember { mutableStateOf("") }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Teslimatı Değerlendir") },
        text = {
            Column {
                Row {
                    (1..5).forEach { value ->
                        Text(
                            if (value <= stars) "★" else "☆",
                            style = MaterialTheme.typography.headlineMedium,
                            color = QervonColors.Warning,
                            modifier = Modifier.padding(2.dp),
                        )
                    }
                }
                OutlinedTextField(value = comment, onValueChange = { comment = it }, label = { Text("Yorum (opsiyonel)") })
            }
        },
        confirmButton = { TextButton(onClick = { onSubmit(stars, comment.ifBlank { null }) }) { Text("Gönder") } },
        dismissButton = { TextButton(onClick = onDismiss) { Text("Vazgeç") } },
    )
}

@Composable
private fun SupportTicketDialog(onDismiss: () -> Unit, onSubmit: (subject: String, message: String) -> Unit) {
    var subject by remember { mutableStateOf("") }
    var message by remember { mutableStateOf("") }

    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Destek Talebi") },
        text = {
            Column {
                OutlinedTextField(value = subject, onValueChange = { subject = it }, label = { Text("Konu") })
                OutlinedTextField(value = message, onValueChange = { message = it }, label = { Text("Mesajınız") })
            }
        },
        confirmButton = {
            TextButton(
                onClick = { if (subject.isNotBlank() && message.isNotBlank()) onSubmit(subject, message) },
            ) { Text("Gönder") }
        },
        dismissButton = { TextButton(onClick = onDismiss) { Text("Vazgeç") } },
    )
}
