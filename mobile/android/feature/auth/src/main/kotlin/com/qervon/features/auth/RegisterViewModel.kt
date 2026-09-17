// =============================================================================
// File:           mobile/android/feature/auth/src/main/kotlin/com/qervon/features/auth/RegisterViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Drives `POST /v1/auth/register` after email and phone OTP checks.
//   Registration returns only a status code — the customer must still log in.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.auth

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.qervon.core.common.QervonApiException
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

data class RegisterUiState(
    val displayName: String = "",
    val email: String = "",
    val phone: String = "",
    val password: String = "",
    val passwordConfirm: String = "",
    val tenantSlug: String = "",
    val emailOtp: String = "",
    val phoneOtp: String = "",
    val codesSent: Boolean = false,
    val emailDevCode: String? = null,
    val phoneDevCode: String? = null,
    val isLoading: Boolean = false,
    val errorMessage: String? = null,
)

sealed class RegisterEvent {
    object Registered : RegisterEvent()
}

@HiltViewModel
class RegisterViewModel @Inject constructor(private val api: QervonApi) : ViewModel() {

    private val _uiState = MutableStateFlow(RegisterUiState())
    val uiState: StateFlow<RegisterUiState> = _uiState.asStateFlow()

    private val _events = MutableSharedFlow<RegisterEvent>(extraBufferCapacity = 1)
    val events: SharedFlow<RegisterEvent> = _events.asSharedFlow()

    fun onDisplayNameChanged(value: String) = update { it.copy(displayName = value) }
    fun onEmailChanged(value: String) = update { it.copy(email = value) }
    fun onPhoneChanged(value: String) = update { it.copy(phone = value) }
    fun onPasswordChanged(value: String) = update { it.copy(password = value) }
    fun onPasswordConfirmChanged(value: String) = update { it.copy(passwordConfirm = value) }
    fun onTenantSlugChanged(value: String) = update { it.copy(tenantSlug = value) }
    fun onEmailOtpChanged(value: String) = update { it.copy(emailOtp = value) }
    fun onPhoneOtpChanged(value: String) = update { it.copy(phoneOtp = value) }

    fun requestCodes() {
        val state = _uiState.value
        if (state.tenantSlug.isBlank() || state.email.isBlank() || state.phone.count { it.isDigit() } < 10) {
            update { it.copy(errorMessage = "Şirket kodu, e-posta ve geçerli bir telefon numarası gerekli.") }
            return
        }
        viewModelScope.launch {
            update { it.copy(isLoading = true, errorMessage = null) }
            try {
                val emailCode = api.requestSignupVerification(state.tenantSlug.trim(), "email", state.email.trim())
                val phoneCode = api.requestSignupVerification(state.tenantSlug.trim(), "sms", state.phone.trim())
                update {
                    it.copy(
                        codesSent = true,
                        emailDevCode = emailCode,
                        phoneDevCode = phoneCode,
                        emailOtp = emailCode.orEmpty().ifBlank { it.emailOtp },
                        phoneOtp = phoneCode.orEmpty().ifBlank { it.phoneOtp },
                    )
                }
            } catch (error: QervonApiException) {
                update { it.copy(errorMessage = error.message) }
            } finally {
                update { it.copy(isLoading = false) }
            }
        }
    }

    fun submit() {
        val state = _uiState.value
        when {
            state.displayName.isBlank() || state.email.isBlank() || state.password.isBlank() -> {
                update { it.copy(errorMessage = "Ad, e-posta ve parola gerekli.") }
                return
            }
            state.tenantSlug.isBlank() -> {
                update { it.copy(errorMessage = "Şirket kodu zorunludur; aksi halde girişte tenant hatası alırsınız.") }
                return
            }
            state.phone.count { it.isDigit() } < 10 -> {
                update { it.copy(errorMessage = "Geçerli bir telefon numarası girin.") }
                return
            }
            state.password.length < 12 -> {
                update { it.copy(errorMessage = "Parola en az 12 karakter olmalı.") }
                return
            }
            state.password != state.passwordConfirm -> {
                update { it.copy(errorMessage = "Parolalar eşleşmiyor.") }
                return
            }
            state.emailOtp.isBlank() || state.phoneOtp.isBlank() -> {
                update { it.copy(errorMessage = "Önce e-posta ve SMS doğrulama kodlarını gönderin.") }
                return
            }
        }
        viewModelScope.launch {
            update { it.copy(isLoading = true, errorMessage = null) }
            try {
                api.register(
                    email = state.email.trim(),
                    displayName = state.displayName.trim(),
                    password = state.password,
                    tenantSlug = state.tenantSlug.trim(),
                    phone = state.phone.trim(),
                    emailOtp = state.emailOtp.trim(),
                    phoneOtp = state.phoneOtp.trim(),
                )
                _events.tryEmit(RegisterEvent.Registered)
            } catch (error: QervonApiException) {
                update { it.copy(errorMessage = error.message) }
            } finally {
                update { it.copy(isLoading = false) }
            }
        }
    }

    private fun update(transform: (RegisterUiState) -> RegisterUiState) {
        _uiState.value = transform(_uiState.value)
    }
}
