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
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material.icons.filled.VisibilityOff
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.text.input.VisualTransformation
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
    var passwordVisible by rememberSaveable { mutableStateOf(false) }
    var confirmVisible by rememberSaveable { mutableStateOf(false) }

    LaunchedEffect(Unit) {
        viewModel.events.collect { event ->
            if (event is RegisterEvent.Registered) onRegistered()
        }
    }

    Scaffold(topBar = { TopAppBar(title = { Text("Hesap oluştur") }) }) { padding ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(padding)
                .verticalScroll(rememberScrollState())
                .padding(QervonSpacing.lg),
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
                value = state.phone,
                onValueChange = viewModel::onPhoneChanged,
                label = { Text("Telefon numarası") },
                singleLine = true,
                keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Phone),
                modifier = Modifier.fillMaxWidth(),
            )
            PasswordField(
                value = state.password,
                onValueChange = viewModel::onPasswordChanged,
                label = "Parola (en az 12 karakter)",
                visible = passwordVisible,
                onToggle = { passwordVisible = !passwordVisible },
            )
            PasswordField(
                value = state.passwordConfirm,
                onValueChange = viewModel::onPasswordConfirmChanged,
                label = "Parola (tekrar)",
                visible = confirmVisible,
                onToggle = { confirmVisible = !confirmVisible },
            )
            OutlinedTextField(
                value = state.tenantSlug,
                onValueChange = viewModel::onTenantSlugChanged,
                label = { Text("Şirket kodu") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            QervonPrimaryButton(
                text = "Doğrulama kodlarını gönder",
                onClick = viewModel::requestCodes,
                isLoading = state.isLoading && !state.codesSent,
            )
            if (state.codesSent) {
                state.emailDevCode?.let { Text("Geliştirme e-posta kodu: $it") }
                state.phoneDevCode?.let { Text("Geliştirme SMS kodu: $it") }
                OutlinedTextField(
                    value = state.emailOtp,
                    onValueChange = viewModel::onEmailOtpChanged,
                    label = { Text("E-posta kodu") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.NumberPassword),
                    modifier = Modifier.fillMaxWidth(),
                )
                OutlinedTextField(
                    value = state.phoneOtp,
                    onValueChange = viewModel::onPhoneOtpChanged,
                    label = { Text("SMS kodu") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.NumberPassword),
                    modifier = Modifier.fillMaxWidth(),
                )
            }
            state.errorMessage?.let {
                Surface(color = QervonColors.Danger.copy(alpha = 0.1f)) {
                    Text(it, color = QervonColors.Danger, modifier = Modifier.padding(QervonSpacing.sm))
                }
            }
            QervonPrimaryButton(text = "Kayıt ol", onClick = viewModel::submit, isLoading = state.isLoading)
        }
    }
}

@Composable
internal fun PasswordField(
    value: String,
    onValueChange: (String) -> Unit,
    label: String,
    visible: Boolean,
    onToggle: () -> Unit,
) {
    OutlinedTextField(
        value = value,
        onValueChange = onValueChange,
        label = { Text(label) },
        singleLine = true,
        visualTransformation = if (visible) VisualTransformation.None else PasswordVisualTransformation(),
        keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Password),
        trailingIcon = {
            IconButton(onClick = onToggle) {
                Icon(
                    imageVector = if (visible) Icons.Filled.VisibilityOff else Icons.Filled.Visibility,
                    contentDescription = if (visible) "Parolayı gizle" else "Parolayı göster",
                )
            }
        },
        modifier = Modifier.fillMaxWidth(),
    )
}
