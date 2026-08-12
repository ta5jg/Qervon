// =============================================================================
// File:           mobile/android/feature/customerprofile/src/main/kotlin/com/qervon/features/customerprofile/CustomerProfileScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.customerprofile

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.qervon.core.designsystem.QervonCard
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonSpacing
import com.qervon.features.addressbook.AddressBookListScreen
import com.qervon.features.addressbook.AddressBookViewModel
import com.qervon.features.addressbook.MapAddressPickerScreen

private enum class ProfileStep { MAIN, ADDRESS_LIST, ADDRESS_PICKER }

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun CustomerProfileScreen(
    onLoggedOut: () -> Unit,
    viewModel: CustomerProfileViewModel = hiltViewModel(),
    addressBookViewModel: AddressBookViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    var step by remember { mutableStateOf(ProfileStep.MAIN) }

    LaunchedEffect(Unit) { viewModel.refresh() }

    if (step == ProfileStep.ADDRESS_LIST) {
        AddressBookListScreen(onAddAddress = { step = ProfileStep.ADDRESS_PICKER })
        return
    }
    if (step == ProfileStep.ADDRESS_PICKER) {
        MapAddressPickerScreen(
            onPicked = { picked ->
                addressBookViewModel.addAddress("Adresim", picked.latitude, picked.longitude, picked.address)
                step = ProfileStep.ADDRESS_LIST
            },
            onClose = { step = ProfileStep.ADDRESS_LIST },
        )
        return
    }

    Scaffold(topBar = { TopAppBar(title = { Text("Hesabım") }) }) { padding ->
        Column(
            modifier = Modifier.fillMaxSize().padding(padding).verticalScroll(rememberScrollState()).padding(QervonSpacing.md),
            verticalArrangement = Arrangement.spacedBy(QervonSpacing.md),
        ) {
            QervonCard {
                Text("Sadakat Puanı", style = MaterialTheme.typography.labelSmall, color = QervonColors.OnSurfaceMuted)
                Text("${state.profile?.loyaltyPoints ?: 0}", style = MaterialTheme.typography.headlineMedium)
            }

            OutlinedButton(onClick = { step = ProfileStep.ADDRESS_LIST }, modifier = Modifier.fillMaxWidth()) {
                Text("Adres Defterim (${state.profile?.addresses?.size ?: 0})")
            }

            QervonCard {
                Text("Destek Taleplerim (${state.tickets.size})", style = MaterialTheme.typography.titleMedium)
                state.tickets.take(3).forEach { ticket ->
                    Text("• ${ticket.subject} — ${ticket.status.displayName()}", color = QervonColors.OnSurfaceMuted)
                }
            }

            QervonCard {
                Text("Bildirimler (${state.notifications.size})", style = MaterialTheme.typography.titleMedium)
                state.notifications.take(3).forEach { notification ->
                    Text("• ${notification.title}", color = QervonColors.OnSurfaceMuted)
                }
            }

            QervonCard {
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
                    Text("Biyometrik Kilit")
                    Switch(checked = state.biometricLockEnabled, onCheckedChange = viewModel::setBiometricLockEnabled)
                }
            }

            state.errorMessage?.let { Text(it, color = QervonColors.Danger) }

            OutlinedButton(onClick = { viewModel.logout(); onLoggedOut() }, modifier = Modifier.fillMaxWidth()) { Text("Çıkış Yap") }
        }
    }
}
