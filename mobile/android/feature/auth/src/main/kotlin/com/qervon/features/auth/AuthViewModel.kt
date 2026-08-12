// =============================================================================
// File:           mobile/android/feature/auth/src/main/kotlin/com/qervon/features/auth/AuthViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Drives both the password and OTP login flows against
//   `POST /v1/auth/login`, `/v1/auth/otp/request`, `/v1/auth/otp/verify`.
//   Tokens are persisted by `QervonApi` itself on success; this
//   ViewModel only tracks UI state and emits a one-shot "authenticated"
//   event for the app's NavHost to react to.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.qervon.core.common.QervonApiException
import com.qervon.core.network.QervonApi
import com.qervon.core.security.AppPreferences
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

enum class LoginMode { PASSWORD, OTP }

data class AuthUiState(
    val mode: LoginMode = LoginMode.PASSWORD,
    val tenantSlug: String = "",
    val email: String = "",
    val password: String = "",
    val phone: String = "",
    val otpCode: String = "",
    val otpRequested: Boolean = false,
    val devCodeHint: String? = null,
    val isLoading: Boolean = false,
    val errorMessage: String? = null,
)

sealed class AuthEvent {
    object Authenticated : AuthEvent()
}

@HiltViewModel
class AuthViewModel @Inject constructor(
    private val api: QervonApi,
    private val preferences: AppPreferences,
) : ViewModel() {

    private val _uiState = MutableStateFlow(AuthUiState(tenantSlug = preferences.lastTenantSlug.orEmpty()))
    val uiState: StateFlow<AuthUiState> = _uiState.asStateFlow()

    private val _events = MutableSharedFlow<AuthEvent>(extraBufferCapacity = 1)
    val events: SharedFlow<AuthEvent> = _events.asSharedFlow()

    fun setMode(mode: LoginMode) {
        _uiState.value = _uiState.value.copy(mode = mode, errorMessage = null, otpRequested = false)
    }

    fun onTenantSlugChanged(value: String) = update { it.copy(tenantSlug = value) }
    fun onEmailChanged(value: String) = update { it.copy(email = value) }
    fun onPasswordChanged(value: String) = update { it.copy(password = value) }
    fun onPhoneChanged(value: String) = update { it.copy(phone = value) }
    fun onOtpCodeChanged(value: String) = update { it.copy(otpCode = value) }

    fun submitPasswordLogin() {
        val state = _uiState.value
        if (state.tenantSlug.isBlank() || state.email.isBlank() || state.password.isBlank()) {
            update { it.copy(errorMessage = "Tüm alanları doldurun.") }
            return
        }
        launchGuarded {
            api.login(state.email.trim(), state.password, state.tenantSlug.trim())
            preferences.lastTenantSlug = state.tenantSlug.trim()
            _events.tryEmit(AuthEvent.Authenticated)
        }
    }

    fun requestOtp() {
        val state = _uiState.value
        if (state.tenantSlug.isBlank() || state.phone.isBlank()) {
            update { it.copy(errorMessage = "Şirket kodu ve telefon numarası gerekli.") }
            return
        }
        launchGuarded {
            val devCode = api.requestOtp(state.tenantSlug.trim(), state.phone.trim())
            update { it.copy(otpRequested = true, devCodeHint = devCode) }
        }
    }

    fun verifyOtp() {
        val state = _uiState.value
        if (state.otpCode.isBlank()) {
            update { it.copy(errorMessage = "Doğrulama kodunu girin.") }
            return
        }
        launchGuarded {
            api.verifyOtp(state.tenantSlug.trim(), state.phone.trim(), state.otpCode.trim())
            preferences.lastTenantSlug = state.tenantSlug.trim()
            _events.tryEmit(AuthEvent.Authenticated)
        }
    }

    private fun update(transform: (AuthUiState) -> AuthUiState) {
        _uiState.value = transform(_uiState.value)
    }

    private fun launchGuarded(block: suspend () -> Unit) {
        viewModelScope.launch {
            update { it.copy(isLoading = true, errorMessage = null) }
            try {
                block()
            } catch (error: QervonApiException) {
                update { it.copy(errorMessage = error.message) }
            } finally {
                update { it.copy(isLoading = false) }
            }
        }
    }
}
