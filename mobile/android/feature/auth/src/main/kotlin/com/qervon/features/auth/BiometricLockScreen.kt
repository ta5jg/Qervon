// =============================================================================
// File:           mobile/android/feature/auth/src/main/kotlin/com/qervon/features/auth/BiometricLockScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Shown on app foreground when the user has previously opted into a
//   biometric app-lock (`AppPreferences.biometricLockEnabled`) and a
//   session already exists. Mirrors the iOS client's `BiometricGate`
//   overlay behaviour.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.auth

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Fingerprint
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing
import com.qervon.core.security.BiometricGate
import androidx.fragment.app.FragmentActivity
import androidx.compose.ui.platform.LocalContext
import kotlinx.coroutines.launch
import androidx.compose.runtime.rememberCoroutineScope

@Composable
fun BiometricLockScreen(appTitle: String, onUnlocked: () -> Unit) {
    // Both apps' MainActivity extend FragmentActivity specifically so
    // BiometricPrompt (which requires a FragmentActivity) works here.
    val activity = LocalContext.current as FragmentActivity
    val scope = rememberCoroutineScope()

    fun promptBiometric() {
        scope.launch {
            val success = BiometricGate.authenticate(activity, appTitle, "Devam etmek için kimliğinizi doğrulayın")
            if (success) onUnlocked()
        }
    }

    LaunchedEffect(Unit) { promptBiometric() }

    Column(
        modifier = Modifier.fillMaxSize().padding(QervonSpacing.lg),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(
            imageVector = Icons.Filled.Fingerprint,
            contentDescription = null,
            tint = QervonColors.Primary,
            modifier = Modifier.padding(bottom = QervonSpacing.md),
        )
        Text(appTitle, style = MaterialTheme.typography.titleLarge)
        Text(
            "Uygulama kilitli",
            style = MaterialTheme.typography.bodyMedium,
            color = QervonColors.OnSurfaceMuted,
            modifier = Modifier.padding(bottom = QervonSpacing.lg),
        )
        QervonPrimaryButton(text = "Tekrar dene", onClick = { promptBiometric() })
    }
}
