// =============================================================================
// File:           mobile/android/feature/auth/src/main/kotlin/com/qervon/features/auth/RegisterScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.auth

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RegisterScreen(
    onRegistered: () -> Unit,
    onBack: () -> Unit,
    viewModel: RegisterViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(Unit) {
        viewModel.events.collect { event ->
            if (event is RegisterEvent.Registered) onRegistered()
        }
    }

    Scaffold(topBar = { TopAppBar(title = { Text("Hesap oluştur") }) }) { padding ->
        Column(
            modifier = Modifier.fillMaxSize().padding(padding).padding(QervonSpacing.lg),
            verticalArrangement = Arrangement.spacedBy(QervonSpacing.md),
        ) {
            OutlinedTextField(
                value = state.displayName,
                onValueChange = viewModel::onDisplayNameChanged,
                label = { Text("Ad Soyad") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = state.email,
                onValueChange = viewModel::onEmailChanged,
                label = { Text("E-posta") },
                singleLine = true,
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Email),
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = state.password,
                onValueChange = viewModel::onPasswordChanged,
                label = { Text("Parola") },
                singleLine = true,
                visualTransformation = PasswordVisualTransformation(),
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = state.tenantSlug,
                onValueChange = viewModel::onTenantSlugChanged,
                label = { Text("Şirket kodu (opsiyonel)") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            state.errorMessage?.let {
                Surface(color = QervonColors.Danger.copy(alpha = 0.1f)) {
                    Text(it, color = QervonColors.Danger, modifier = Modifier.padding(QervonSpacing.sm))
                }
            }
            QervonPrimaryButton(text = "Kayıt ol", onClick = viewModel::submit, isLoading = state.isLoading)
        }
    }
}
