// =============================================================================
// File:           mobile/android/feature/auth/src/main/kotlin/com/qervon/features/auth/LoginScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Password-or-OTP login screen shared by both apps. [appTitle],
//   [appSubtitle], and [showsRegistration] let each app brand this
//   screen without needing its own copy.
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
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun LoginScreen(
    appTitle: String,
    appSubtitle: String,
    showsRegistration: Boolean,
    onAuthenticated: () -> Unit,
    onNavigateToRegister: () -> Unit,
    viewModel: AuthViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(Unit) {
        viewModel.events.collect { event ->
            if (event is AuthEvent.Authenticated) onAuthenticated()
        }
    }

    Scaffold { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(QervonSpacing.lg),
            verticalArrangement = Arrangement.spacedBy(QervonSpacing.md),
        ) {
            Text(appTitle, style = MaterialTheme.typography.headlineMedium)
            Text(appSubtitle, style = MaterialTheme.typography.bodyMedium, color = QervonColors.OnSurfaceMuted)

            SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
                SegmentedButton(
                    selected = state.mode == LoginMode.PASSWORD,
                    onClick = { viewModel.setMode(LoginMode.PASSWORD) },
                    shape = SegmentedButtonDefaults.itemShape(0, 2),
                ) { Text("Parola") }
                SegmentedButton(
                    selected = state.mode == LoginMode.OTP,
                    onClick = { viewModel.setMode(LoginMode.OTP) },
                    shape = SegmentedButtonDefaults.itemShape(1, 2),
                ) { Text("SMS Kodu") }
            }

            OutlinedTextField(
                value = state.tenantSlug,
                onValueChange = viewModel::onTenantSlugChanged,
                label = { Text("Şirket kodu") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )

            if (state.mode == LoginMode.PASSWORD) {
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
                QervonPrimaryButton(text = "Giriş yap", onClick = viewModel::submitPasswordLogin, isLoading = state.isLoading)
            } else {
                OutlinedTextField(
                    value = state.phone,
                    onValueChange = viewModel::onPhoneChanged,
                    label = { Text("Telefon numarası") },
                    singleLine = true,
                    enabled = !state.otpRequested,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Phone),
                    modifier = Modifier.fillMaxWidth(),
                )
                if (!state.otpRequested) {
                    QervonPrimaryButton(text = "Kod gönder", onClick = viewModel::requestOtp, isLoading = state.isLoading)
                } else {
                    state.devCodeHint?.let {
                        Text("Geliştirme kodu: $it", style = MaterialTheme.typography.labelSmall, color = QervonColors.Warning)
                    }
                    OutlinedTextField(
                        value = state.otpCode,
                        onValueChange = viewModel::onOtpCodeChanged,
                        label = { Text("Doğrulama kodu") },
                        singleLine = true,
                        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.NumberPassword),
                        modifier = Modifier.fillMaxWidth(),
                    )
                    QervonPrimaryButton(text = "Doğrula", onClick = viewModel::verifyOtp, isLoading = state.isLoading)
                }
            }

            state.errorMessage?.let {
                Surface(color = QervonColors.Danger.copy(alpha = 0.1f)) {
                    Text(it, color = QervonColors.Danger, modifier = Modifier.padding(QervonSpacing.sm))
                }
            }

            if (showsRegistration) {
                TextButton(onClick = onNavigateToRegister, modifier = Modifier.fillMaxWidth()) {
                    Text("Hesabınız yok mu? Kayıt olun")
                }
            }
        }
    }
}
