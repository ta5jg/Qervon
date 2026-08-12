// =============================================================================
// File:           mobile/android/app-courier/src/main/kotlin/com/qervon/android/courier/CourierRootViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Tracks whether a session exists (a token was ever saved) and whether
//   the app is currently biometric-locked, independent of any single
//   screen's lifecycle — the composition root's minimal state machine,
//   mirroring the iOS client's `AppSession`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.android.courier

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
class CourierRootViewModel @Inject constructor(
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
