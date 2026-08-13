// =============================================================================
// File:           mobile/android/feature/customerprofile/src/main/kotlin/com/qervon/features/customerprofile/CustomerSupportScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-13
// Version:        0.1.0
//
// Description:
//   Customer support center tab: ticket thread list and ticket creation form.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.customerprofile

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.qervon.core.common.QervonFormat
import com.qervon.core.common.model.TicketStatus
import com.qervon.core.designsystem.QervonCard
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing
import com.qervon.core.designsystem.StatusPill

@Composable
fun CustomerSupportScreen(
    viewModel: CustomerSupportViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    var subject by remember { mutableStateOf("Mobil Destek Talebi") }
    var message by remember { mutableStateOf("") }

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
                Text("Yeni Destek Talebi", style = MaterialTheme.typography.titleMedium)
                OutlinedTextField(
                    value = subject,
                    onValueChange = { subject = it },
                    label = { Text("Konu") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth().padding(top = QervonSpacing.sm),
                )
                OutlinedTextField(
                    value = message,
                    onValueChange = { message = it },
                    label = { Text("Mesaj") },
                    modifier = Modifier.fillMaxWidth().padding(top = QervonSpacing.sm),
                )
                QervonPrimaryButton(
                    text = "Talep Oluştur",
                    onClick = {
                        viewModel.submitTicket(subject, message)
                        message = ""
                    },
                    enabled = subject.isNotBlank() && message.isNotBlank() && !state.isSubmitting,
                    isLoading = state.isSubmitting,
                    modifier = Modifier.padding(top = QervonSpacing.sm),
                )
                state.infoMessage?.let {
                    Text(
                        it,
                        color = QervonColors.Success,
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.padding(top = QervonSpacing.sm),
                    )
                }
                state.errorMessage?.let {
                    Text(
                        it,
                        color = QervonColors.Danger,
                        style = MaterialTheme.typography.bodySmall,
                        modifier = Modifier.padding(top = QervonSpacing.sm),
                    )
                }
            }
        }

        if (state.tickets.isEmpty() && !state.isLoading) {
            item {
                QervonCard {
                    Text("Henüz destek talebiniz yok.", color = QervonColors.OnSurfaceMuted)
                }
            }
        } else {
            items(state.tickets, key = { it.id }) { ticket ->
                QervonCard {
                    Text(ticket.subject, style = MaterialTheme.typography.titleSmall)
                    Text(
                        ticket.message,
                        style = MaterialTheme.typography.bodyMedium,
                        color = QervonColors.OnSurfaceMuted,
                        modifier = Modifier.padding(top = QervonSpacing.xs),
                    )
                    androidx.compose.foundation.layout.Row(
                        modifier = Modifier.fillMaxWidth().padding(top = QervonSpacing.sm),
                        horizontalArrangement = Arrangement.SpaceBetween,
                    ) {
                        Text(
                            QervonFormat.dayAndTime(ticket.createdAt),
                            style = MaterialTheme.typography.labelSmall,
                            color = QervonColors.OnSurfaceMuted,
                        )
                        StatusPill(
                            text = ticket.status.displayName(),
                            color = when (ticket.status) {
                                TicketStatus.OPEN -> QervonColors.Warning
                                TicketStatus.IN_PROGRESS -> QervonColors.Primary
                                TicketStatus.RESOLVED -> QervonColors.Success
                                TicketStatus.CLOSED -> QervonColors.OnSurfaceMuted
                            },
                        )
                    }
                }
            }
        }
    }
}
