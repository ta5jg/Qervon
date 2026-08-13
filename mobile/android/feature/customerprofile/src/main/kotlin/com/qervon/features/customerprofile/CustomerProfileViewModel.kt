// =============================================================================
// File:           mobile/android/feature/customerprofile/src/main/kotlin/com/qervon/features/customerprofile/CustomerProfileViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Loads the customer profile, support tickets, and notifications
//   (`GET /v1/customer/profile`, `/support-tickets`, `/notifications`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.customerprofile

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.qervon.core.common.QervonApiException
import com.qervon.core.common.model.AppNotification
import com.qervon.core.common.model.CustomerProfile
import com.qervon.core.common.model.SupportTicket
import com.qervon.core.network.QervonApi
import com.qervon.core.security.AppPreferences
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import javax.inject.Inject

private const val SUPPORT_POLL_INTERVAL_MS = 5_000L

data class CustomerProfileUiState(
    val profile: CustomerProfile? = null,
    val tickets: List<SupportTicket> = emptyList(),
    val notifications: List<AppNotification> = emptyList(),
    val biometricLockEnabled: Boolean = false,
    val isLoading: Boolean = false,
    val isSubmittingSupportTicket: Boolean = false,
    val errorMessage: String? = null,
    val infoMessage: String? = null,
)

@HiltViewModel
class CustomerProfileViewModel @Inject constructor(
    private val api: QervonApi,
    private val preferences: AppPreferences,
) : ViewModel() {

    private val _uiState = MutableStateFlow(CustomerProfileUiState(biometricLockEnabled = preferences.biometricLockEnabled))
    val uiState: StateFlow<CustomerProfileUiState> = _uiState.asStateFlow()
    private var supportPollJob: Job? = null

    fun refresh() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true, errorMessage = null, infoMessage = null)
            try {
                val profile = api.getCustomerProfile()
                val tickets = api.listSupportTickets()
                val notifications = api.listNotifications()
                _uiState.value = _uiState.value.copy(profile = profile, tickets = tickets, notifications = notifications)
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isLoading = false)
            }
        }
    }

    fun startLiveSupport() {
        if (supportPollJob?.isActive == true) return
        supportPollJob = viewModelScope.launch {
            while (isActive) {
                delay(SUPPORT_POLL_INTERVAL_MS)
                refreshSupportTickets()
            }
        }
    }

    fun stopLiveSupport() {
        supportPollJob?.cancel()
        supportPollJob = null
    }

    fun submitSupportTicket(subject: String, message: String) {
        val trimmedSubject = subject.trim()
        val trimmedMessage = message.trim()
        if (trimmedSubject.isEmpty() || trimmedMessage.isEmpty()) return

        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(
                isSubmittingSupportTicket = true,
                errorMessage = null,
                infoMessage = null,
            )
            try {
                api.createSupportTicket(orderId = null, subject = trimmedSubject, message = trimmedMessage)
                val tickets = api.listSupportTickets()
                _uiState.value = _uiState.value.copy(
                    tickets = tickets,
                    infoMessage = "Destek talebiniz operatore iletildi.",
                )
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isSubmittingSupportTicket = false)
            }
        }
    }

    private suspend fun refreshSupportTickets() {
        try {
            val tickets = api.listSupportTickets()
            _uiState.value = _uiState.value.copy(tickets = tickets)
        } catch (_: QervonApiException) {
            // Keep existing tickets if a transient poll refresh fails.
        }
    }

    fun setBiometricLockEnabled(enabled: Boolean) {
        preferences.biometricLockEnabled = enabled
        _uiState.value = _uiState.value.copy(biometricLockEnabled = enabled)
    }

    fun logout() {
        api.logout()
    }

    override fun onCleared() {
        stopLiveSupport()
        super.onCleared()
    }
}
