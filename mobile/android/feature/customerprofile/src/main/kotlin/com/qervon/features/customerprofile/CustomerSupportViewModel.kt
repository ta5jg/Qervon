// =============================================================================
// File:           mobile/android/feature/customerprofile/src/main/kotlin/com/qervon/features/customerprofile/CustomerSupportViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-13
// Version:        0.1.0
//
// Description:
//   Dedicated customer support center state: list/create support tickets with
//   periodic live refresh.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.customerprofile

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.qervon.core.common.QervonApiException
import com.qervon.core.common.model.SupportTicket
import com.qervon.core.network.QervonApi
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import javax.inject.Inject

private const val SUPPORT_CENTER_POLL_INTERVAL_MS = 5_000L

data class CustomerSupportUiState(
    val tickets: List<SupportTicket> = emptyList(),
    val isLoading: Boolean = false,
    val isSubmitting: Boolean = false,
    val infoMessage: String? = null,
    val errorMessage: String? = null,
)

@HiltViewModel
class CustomerSupportViewModel @Inject constructor(
    private val api: QervonApi,
) : ViewModel() {

    private val _uiState = MutableStateFlow(CustomerSupportUiState())
    val uiState: StateFlow<CustomerSupportUiState> = _uiState.asStateFlow()

    private var pollJob: Job? = null

    fun refresh() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true, errorMessage = null)
            try {
                val tickets = api.listSupportTickets()
                _uiState.value = _uiState.value.copy(tickets = tickets)
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isLoading = false)
            }
        }
    }

    fun startLiveUpdates() {
        if (pollJob?.isActive == true) return
        pollJob = viewModelScope.launch {
            while (isActive) {
                delay(SUPPORT_CENTER_POLL_INTERVAL_MS)
                try {
                    val tickets = api.listSupportTickets()
                    _uiState.value = _uiState.value.copy(tickets = tickets)
                } catch (_: QervonApiException) {
                    // Keep last successful list on transient poll failures.
                }
            }
        }
    }

    fun stopLiveUpdates() {
        pollJob?.cancel()
        pollJob = null
    }

    fun submitTicket(subject: String, message: String) {
        val trimmedSubject = subject.trim()
        val trimmedMessage = message.trim()
        if (trimmedSubject.isEmpty() || trimmedMessage.isEmpty()) return

        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isSubmitting = true,
                errorMessage = null,
                infoMessage = null,
            )
            try {
                api.createSupportTicket(
                    orderId = null,
                    subject = trimmedSubject,
                    message = trimmedMessage,
                )
                val tickets = api.listSupportTickets()
                _uiState.value = _uiState.value.copy(
                    tickets = tickets,
                    infoMessage = "Destek talebiniz iletildi.",
                )
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isSubmitting = false)
            }
        }
    }

    override fun onCleared() {
        stopLiveUpdates()
        super.onCleared()
    }
}
