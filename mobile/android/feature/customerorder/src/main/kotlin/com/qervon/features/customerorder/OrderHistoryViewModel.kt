// =============================================================================
// File:           mobile/android/feature/customerorder/src/main/kotlin/com/qervon/features/customerorder/OrderHistoryViewModel.kt
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

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.qervon.core.common.QervonApiException
import com.qervon.core.common.model.Order
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

private const val ORDER_HISTORY_POLL_INTERVAL_MS = 5_000L

data class OrderHistoryUiState(
    val orders: List<Order> = emptyList(),
    val isLoading: Boolean = false,
    val errorMessage: String? = null,
)

@HiltViewModel
class OrderHistoryViewModel @Inject constructor(private val api: QervonApi) : ViewModel() {

    private val _uiState = MutableStateFlow(OrderHistoryUiState())
    val uiState: StateFlow<OrderHistoryUiState> = _uiState.asStateFlow()
    private var pollJob: Job? = null

    fun refresh() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true, errorMessage = null)
            try {
                val orders = api.listCustomerOrders().sortedByDescending { it.createdAt }
                _uiState.value = _uiState.value.copy(orders = orders)
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
                delay(ORDER_HISTORY_POLL_INTERVAL_MS)
                try {
                    val orders = api.listCustomerOrders().sortedByDescending { it.createdAt }
                    _uiState.value = _uiState.value.copy(orders = orders)
                } catch (_: QervonApiException) {
                    // Keep previous list on transient refresh errors.
                }
            }
        }
    }

    fun stopLiveUpdates() {
        pollJob?.cancel()
        pollJob = null
    }

    override fun onCleared() {
        stopLiveUpdates()
        super.onCleared()
    }
}
