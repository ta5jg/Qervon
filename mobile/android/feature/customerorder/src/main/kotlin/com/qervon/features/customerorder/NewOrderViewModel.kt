// =============================================================================
// File:           mobile/android/feature/customerorder/src/main/kotlin/com/qervon/features/customerorder/NewOrderViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Drives the new-order screen: as soon as both pickup and dropoff are
//   set, fetches a live, non-binding fare estimate
//   (`GET /v1/customer/fare-quote`) and, on submit, creates the order
//   (`POST /v1/customer/orders`) which recomputes the authoritative fare
//   server-side.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.customerorder

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.qervon.core.common.QervonApiException
import com.qervon.core.common.model.Address
import com.qervon.core.common.model.FareQuote
import com.qervon.core.common.model.Order
import com.qervon.core.network.QervonApi
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

data class NewOrderUiState(
    val pickup: Address? = null,
    val dropoff: Address? = null,
    val couponCode: String = "",
    val paymentMethod: String = "cash",
    val deliveryNote: String = "",
    val contactPhone: String = "",
    val fareQuote: FareQuote? = null,
    val isQuoting: Boolean = false,
    val isSubmitting: Boolean = false,
    val errorMessage: String? = null,
) {
    val canSubmit: Boolean get() = pickup != null && dropoff != null && !isSubmitting
}

sealed class NewOrderEvent {
    data class Created(val order: Order) : NewOrderEvent()
}

@HiltViewModel
class NewOrderViewModel @Inject constructor(private val api: QervonApi) : ViewModel() {

    private val _uiState = MutableStateFlow(NewOrderUiState())
    val uiState: StateFlow<NewOrderUiState> = _uiState.asStateFlow()

    private val _events = MutableSharedFlow<NewOrderEvent>(extraBufferCapacity = 1)
    val events: SharedFlow<NewOrderEvent> = _events.asSharedFlow()

    fun setPickup(address: Address) {
        _uiState.value = _uiState.value.copy(pickup = address)
        refreshQuoteIfPossible()
    }

    fun setDropoff(address: Address) {
        _uiState.value = _uiState.value.copy(dropoff = address)
        refreshQuoteIfPossible()
    }

    fun onCouponCodeChanged(value: String) {
        _uiState.value = _uiState.value.copy(couponCode = value)
    }

    fun onPaymentMethodChanged(value: String) {
        _uiState.value = _uiState.value.copy(paymentMethod = value)
    }

    fun onDeliveryNoteChanged(value: String) {
        _uiState.value = _uiState.value.copy(deliveryNote = value)
    }

    fun onContactPhoneChanged(value: String) {
        _uiState.value = _uiState.value.copy(contactPhone = value)
    }

    private fun refreshQuoteIfPossible() {
        val state = _uiState.value
        val pickup = state.pickup ?: return
        val dropoff = state.dropoff ?: return
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isQuoting = true)
            try {
                val quote = api.getFareQuote(pickup, dropoff)
                _uiState.value = _uiState.value.copy(fareQuote = quote)
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isQuoting = false)
            }
        }
    }

    fun submit() {
        val state = _uiState.value
        val pickup = state.pickup ?: return
        val dropoff = state.dropoff ?: return
        viewModelScope.launch {
            _uiState.value = state.copy(isSubmitting = true, errorMessage = null)
            try {
                val order = api.createOrder(
                    pickup = pickup,
                    dropoff = dropoff,
                    couponCode = state.couponCode.trim().ifBlank { null },
                    paymentMethod = state.paymentMethod,
                    deliveryNote = state.deliveryNote.trim().ifBlank { null },
                    contactPhone = state.contactPhone.trim().ifBlank { null },
                )
                _events.tryEmit(NewOrderEvent.Created(order))
                _uiState.value = NewOrderUiState()
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isSubmitting = false)
            }
        }
    }
}
