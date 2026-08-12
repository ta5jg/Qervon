// =============================================================================
// File:           mobile/android/feature/orders/src/main/kotlin/com/qervon/features/orders/OrdersViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Lists the courier's active jobs (`GET /v1/courier/orders`) and drives
//   the pickup transition (`POST /v1/courier/orders/{id}/pickup`) —
//   delivery itself happens in `feature:proof` since it requires
//   QR/signature/photo evidence.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.orders

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.qervon.core.common.QervonApiException
import com.qervon.core.common.model.Order
import com.qervon.core.common.model.OrderStatus
import com.qervon.core.network.QervonApi
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

data class OrdersUiState(
    val orders: List<Order> = emptyList(),
    val isLoading: Boolean = false,
    val errorMessage: String? = null,
    val processingOrderId: String? = null,
)

@HiltViewModel
class OrdersViewModel @Inject constructor(private val api: QervonApi) : ViewModel() {

    private val _uiState = MutableStateFlow(OrdersUiState())
    val uiState: StateFlow<OrdersUiState> = _uiState.asStateFlow()

    fun refresh() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true, errorMessage = null)
            try {
                val activeOrders = api.listCourierOrders().filter {
                    it.status == OrderStatus.COURIER_ASSIGNED || it.status == OrderStatus.IN_TRANSIT
                }
                _uiState.value = _uiState.value.copy(orders = activeOrders)
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isLoading = false)
            }
        }
    }

    fun pickup(orderId: String) {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(processingOrderId = orderId, errorMessage = null)
            try {
                api.pickupOrder(orderId)
                refresh()
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(processingOrderId = null)
            }
        }
    }
}
