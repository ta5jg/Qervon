// =============================================================================
// File:           mobile/android/feature/auth/src/main/kotlin/com/qervon/features/auth/RegisterViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Drives `POST /v1/auth/register` (customer-only, per the backend's
//   `auth_register` handler). Registration returns only a status code —
//   the customer must still log in afterwards.
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
    val password: String = "",
    val tenantSlug: String = "",
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
    fun onPasswordChanged(value: String) = update { it.copy(password = value) }
    fun onTenantSlugChanged(value: String) = update { it.copy(tenantSlug = value) }

    fun submit() {
        val state = _uiState.value
        if (state.displayName.isBlank() || state.email.isBlank() || state.password.isBlank()) {
            update { it.copy(errorMessage = "Ad, e-posta ve parola gerekli.") }
            return
        }
        viewModelScope.launch {
            update { it.copy(isLoading = true, errorMessage = null) }
            try {
                api.register(
                    email = state.email.trim(),
                    displayName = state.displayName.trim(),
                    password = state.password,
                    tenantSlug = state.tenantSlug.trim().ifBlank { null },
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
