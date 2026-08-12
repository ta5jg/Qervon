// =============================================================================
// File:           mobile/android/app-customer/src/main/kotlin/com/qervon/android/customer/CustomerRootViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   See app-courier's `CourierRootViewModel.kt` for the full rationale —
//   identical state machine, duplicated per-app.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.android.customer

import androidx.lifecycle.ViewModel
import com.qervon.core.common.AuthTokenStore
import com.qervon.core.security.AppPreferences
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import javax.inject.Inject

data class RootState(
    val hasSession: Boolean,
    val isUnlocked: Boolean,
)

@HiltViewModel
class CustomerRootViewModel @Inject constructor(
    private val tokenStore: AuthTokenStore,
    private val preferences: AppPreferences,
) : ViewModel() {

    private val hasSessionAtLaunch = tokenStore.currentTokens() != null

    private val _state = MutableStateFlow(
        RootState(
            hasSession = hasSessionAtLaunch,
            isUnlocked = !(hasSessionAtLaunch && preferences.biometricLockEnabled),
        ),
    )
    val state: StateFlow<RootState> = _state.asStateFlow()

    fun onAuthenticated() {
        _state.value = _state.value.copy(hasSession = true, isUnlocked = true)
    }

    fun onBiometricUnlocked() {
        _state.value = _state.value.copy(isUnlocked = true)
    }

    fun onLoggedOut() {
        tokenStore.clear()
        _state.value = _state.value.copy(hasSession = false)
    }
}
