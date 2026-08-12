// =============================================================================
// File:           mobile/android/feature/customerorder/src/main/kotlin/com/qervon/features/customerorder/OrderDetailViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Polls `GET /v1/orders/{id}/tracking` and `GET /v1/customer/orders/{id}/eta`
//   every 5s while the order is `courier_assigned`/`in_transit`, and drives
//   cancel (`POST .../cancel`), rating (`POST .../rating`), and support
//   ticket creation.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.customerorder

import androidx.lifecycle.SavedStateHandle
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.qervon.core.common.QervonApiException
import com.qervon.core.common.model.EtaInfo
import com.qervon.core.common.model.LocationSnapshot
import com.qervon.core.common.model.Order
import com.qervon.core.common.model.OrderStatus
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

private const val POLL_INTERVAL_MS = 5_000L

data class OrderDetailUiState(
    val order: Order? = null,
    val courierLocation: LocationSnapshot? = null,
    val eta: EtaInfo? = null,
    val isLoading: Boolean = false,
    val isSubmittingAction: Boolean = false,
    val errorMessage: String? = null,
    val infoMessage: String? = null,
) {
    val canCancel: Boolean get() = order?.status == OrderStatus.PENDING || order?.status == OrderStatus.COURIER_ASSIGNED
    val canRate: Boolean get() = order?.status == OrderStatus.DELIVERED
    val isTrackable: Boolean get() = order?.status == OrderStatus.COURIER_ASSIGNED || order?.status == OrderStatus.IN_TRANSIT
}

@HiltViewModel
class OrderDetailViewModel @Inject constructor(
    private val api: QervonApi,
    savedStateHandle: SavedStateHandle,
) : ViewModel() {

    private val orderId: String = savedStateHandle.get<String>("orderId").orEmpty()

    private val _uiState = MutableStateFlow(OrderDetailUiState())
    val uiState: StateFlow<OrderDetailUiState> = _uiState.asStateFlow()

    private var pollJob: Job? = null

    fun start() {
        refresh()
        if (pollJob?.isActive != true) {
            pollJob = viewModelScope.launch {
                while (isActive) {
                    delay(POLL_INTERVAL_MS)
                    refresh(showLoading = false)
                }
            }
        }
    }

    override fun onCleared() {
        pollJob?.cancel()
        super.onCleared()
    }

    private fun refresh(showLoading: Boolean = true) {
        viewModelScope.launch {
            if (showLoading) _uiState.value = _uiState.value.copy(isLoading = true)
            try {
                val orders = api.listCustomerOrders()
                val order = orders.firstOrNull { it.id == orderId }
                _uiState.value = _uiState.value.copy(order = order)
                if (order != null && (order.status == OrderStatus.COURIER_ASSIGNED || order.status == OrderStatus.IN_TRANSIT)) {
                    val eta = runCatching { api.getOrderEta(orderId) }.getOrNull()
                    val location = runCatching { api.getOrderTracking(orderId) }.getOrNull()
                    _uiState.value = _uiState.value.copy(eta = eta, courierLocation = location)
                }
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isLoading = false)
            }
        }
    }

    fun cancelOrder() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isSubmittingAction = true, errorMessage = null)
            try {
                val order = api.cancelOrder(orderId)
                _uiState.value = _uiState.value.copy(order = order, infoMessage = "Sipariş iptal edildi.")
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isSubmittingAction = false)
            }
        }
    }

    fun submitRating(stars: Int, comment: String?) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isSubmittingAction = true, errorMessage = null)
            try {
                api.rateOrder(orderId, stars, comment)
                _uiState.value = _uiState.value.copy(infoMessage = "Değerlendirmeniz için teşekkürler.")
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isSubmittingAction = false)
            }
        }
    }

    fun submitSupportTicket(subject: String, message: String) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isSubmittingAction = true, errorMessage = null)
            try {
                api.createSupportTicket(orderId, subject, message)
                _uiState.value = _uiState.value.copy(infoMessage = "Destek talebiniz iletildi.")
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isSubmittingAction = false)
            }
        }
    }
}
