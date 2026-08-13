// =============================================================================
// File:           mobile/android/feature/dispatch/src/main/kotlin/com/qervon/features/dispatch/DispatchViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Owns the courier's online/offline toggle
//   (`POST /v1/courier/me/status`), starts/stops the foreground location
//   service, and polls `GET /v1/courier/me/offer` every 4s while online
//   for an incoming job to accept/reject.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.dispatch

import android.content.Context
import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.qervon.core.common.QervonApiException
import com.qervon.core.common.model.Courier
import com.qervon.core.common.model.CourierStatus
import com.qervon.core.common.model.Order
import com.qervon.core.common.model.OrderStatus
import com.qervon.core.common.model.PendingOffer
import com.qervon.core.location.CourierLocationService
import com.qervon.core.location.LocationReporter
import com.qervon.core.network.QervonApi
import com.qervon.core.security.AppPreferences
import dagger.hilt.android.lifecycle.HiltViewModel
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import javax.inject.Inject

private const val OFFER_POLL_INTERVAL_MS = 4_000L

data class DispatchUiState(
    val courier: Courier? = null,
    val activeOrder: Order? = null,
    val pendingOffer: PendingOffer? = null,
    val secondsRemaining: Long = 0,
    val isTogglingOnline: Boolean = false,
    val isRespondingToOffer: Boolean = false,
    val errorMessage: String? = null,
) {
    val isOnline: Boolean get() = courier?.status != null && courier.status != CourierStatus.OFFLINE
}

@HiltViewModel
class DispatchViewModel @Inject constructor(
    private val api: QervonApi,
    private val preferences: AppPreferences,
    @ApplicationContext private val appContext: Context,
) : ViewModel() {

    private val _uiState = MutableStateFlow(DispatchUiState())
    val uiState: StateFlow<DispatchUiState> = _uiState.asStateFlow()

    private var pollJob: Job? = null

    init {
        CourierLocationService.reporter = object : LocationReporter {
            override suspend fun reportLocation(latitude: Double, longitude: Double, speedKmh: Double?, batteryPct: Int?) {
                try {
                    api.updateOwnLocation(latitude, longitude, speedKmh, batteryPct)
                } catch (_: QervonApiException) {
                    // Best-effort: a single missed location beat is not
                    // surfaced to the courier, it will retry on the next tick.
                }
            }
        }
        refreshCourier()
    }

    fun refreshCourier() {
        viewModelScope.launch {
            try {
                val courier = api.getOwnCourier()
                val activeOrder = api.listCourierOrders().firstOrNull {
                    it.status == OrderStatus.COURIER_ASSIGNED || it.status == OrderStatus.IN_TRANSIT
                }
                _uiState.value = _uiState.value.copy(
                    courier = courier,
                    activeOrder = activeOrder,
                )
                if (courier.status != CourierStatus.OFFLINE) startPolling()
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            }
        }
    }

    fun toggleOnline() {
        val goingOnline = !_uiState.value.isOnline
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isTogglingOnline = true, errorMessage = null)
            try {
                val courier = api.setOwnAvailability(goingOnline)
                _uiState.value = _uiState.value.copy(courier = courier)
                preferences.courierOnlineOnAppStart = goingOnline
                if (goingOnline) {
                    CourierLocationService.start(appContext)
                    startPolling()
                } else {
                    CourierLocationService.stop(appContext)
                    stopPolling()
                    _uiState.value = _uiState.value.copy(pendingOffer = null, activeOrder = null)
                }
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isTogglingOnline = false)
            }
        }
    }

    fun acceptOffer() {
        val offer = _uiState.value.pendingOffer ?: return
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isRespondingToOffer = true)
            try {
                val accepted = api.acceptOffer(offer.order.id)
                _uiState.value = _uiState.value.copy(pendingOffer = null, activeOrder = accepted)
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isRespondingToOffer = false)
            }
        }
    }

    fun rejectOffer() {
        val offer = _uiState.value.pendingOffer ?: return
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isRespondingToOffer = true)
            try {
                api.rejectOffer(offer.order.id)
                _uiState.value = _uiState.value.copy(pendingOffer = null)
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isRespondingToOffer = false)
            }
        }
    }

    private fun startPolling() {
        if (pollJob?.isActive == true) return
        pollJob = viewModelScope.launch {
            while (isActive) {
                try {
                    val activeOrder = api.listCourierOrders().firstOrNull {
                        it.status == OrderStatus.COURIER_ASSIGNED || it.status == OrderStatus.IN_TRANSIT
                    }
                    if (activeOrder != null) {
                        _uiState.value = _uiState.value.copy(
                            activeOrder = activeOrder,
                            pendingOffer = null,
                            secondsRemaining = 0,
                        )
                    } else {
                        val offer = api.getOwnPendingOffer()
                        _uiState.value = _uiState.value.copy(
                            activeOrder = null,
                            pendingOffer = offer,
                            secondsRemaining = offer?.secondsRemaining() ?: 0,
                        )
                    }
                } catch (_: QervonApiException) {
                    // Transient poll failures are silently retried.
                }
                delay(OFFER_POLL_INTERVAL_MS)
            }
        }
    }

    private fun stopPolling() {
        pollJob?.cancel()
        pollJob = null
    }

    override fun onCleared() {
        stopPolling()
        super.onCleared()
    }
}
