// =============================================================================
// File:           mobile/android/feature/profile/src/main/kotlin/com/qervon/features/profile/CourierProfileViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Courier account screen state: profile summary, phone binding
//   (`POST /v1/auth/phone`, required before OTP login works — see
//   `set_own_phone` in the backend), biometric-lock preference, logout.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.profile

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.qervon.core.common.QervonApiException
import com.qervon.core.common.model.Courier
import com.qervon.core.network.QervonApi
import com.qervon.core.security.AppPreferences
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

data class CourierProfileUiState(
    val courier: Courier? = null,
    val phoneInput: String = "",
    val phoneBound: Boolean = false,
    val biometricLockEnabled: Boolean = false,
    val isLoading: Boolean = false,
    val errorMessage: String? = null,
    val infoMessage: String? = null,
)

@HiltViewModel
class CourierProfileViewModel @Inject constructor(
    private val api: QervonApi,
    private val preferences: AppPreferences,
) : ViewModel() {

    private val _uiState = MutableStateFlow(CourierProfileUiState(biometricLockEnabled = preferences.biometricLockEnabled))
    val uiState: StateFlow<CourierProfileUiState> = _uiState.asStateFlow()

    fun refresh() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true, errorMessage = null)
            try {
                val courier = api.getOwnCourier()
                _uiState.value = _uiState.value.copy(courier = courier)
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isLoading = false)
            }
        }
    }

    fun onPhoneInputChanged(value: String) {
        _uiState.value = _uiState.value.copy(phoneInput = value)
    }

    fun bindPhone() {
        val phone = _uiState.value.phoneInput.trim()
        if (phone.isBlank()) return
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true, errorMessage = null, infoMessage = null)
            try {
                api.setPhone(phone)
                _uiState.value = _uiState.value.copy(phoneBound = true, infoMessage = "Telefon numarası kaydedildi.")
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isLoading = false)
            }
        }
    }

    fun setBiometricLockEnabled(enabled: Boolean) {
        preferences.biometricLockEnabled = enabled
        _uiState.value = _uiState.value.copy(biometricLockEnabled = enabled)
    }

    fun logout() {
        api.logout()
    }
}
