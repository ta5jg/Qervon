// =============================================================================
// File:           mobile/android/feature/profile/src/main/kotlin/com/qervon/features/profile/CourierProfileScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.profile

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.qervon.core.designsystem.QervonCard
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing

@Composable
fun CourierProfileScreen(onLoggedOut: () -> Unit, viewModel: CourierProfileViewModel = hiltViewModel()) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(Unit) { viewModel.refresh() }

    Column(
        modifier = Modifier.fillMaxSize().padding(QervonSpacing.md),
        verticalArrangement = Arrangement.spacedBy(QervonSpacing.md),
    ) {
        QervonCard {
            Text(state.courier?.name ?: "—", style = MaterialTheme.typography.titleLarge)
            Text(state.courier?.vehicle?.displayName() ?: "—", color = QervonColors.OnSurfaceMuted)
        }

        QervonCard {
            Text("Telefon Numarası (SMS girişi için gerekli)", style = MaterialTheme.typography.titleMedium)
            OutlinedTextField(
                value = state.phoneInput,
                onValueChange = viewModel::onPhoneInputChanged,
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
            )
            QervonPrimaryButton(text = "Kaydet", onClick = viewModel::bindPhone, isLoading = state.isLoading)
        }

        QervonCard {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                Text("Biyometrik Kilit")
                Switch(checked = state.biometricLockEnabled, onCheckedChange = viewModel::setBiometricLockEnabled)
            }
        }

        state.infoMessage?.let { Text(it, color = QervonColors.Success) }
        state.errorMessage?.let { Text(it, color = QervonColors.Danger) }

        OutlinedButton(
            onClick = { viewModel.logout(); onLoggedOut() },
            modifier = Modifier.fillMaxWidth(),
        ) { Text("Çıkış Yap") }
    }
}
