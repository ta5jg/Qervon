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
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
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

@Composable
fun CustomerProfileScreen(
    onLoggedOut: () -> Unit,
    viewModel: CustomerProfileViewModel = hiltViewModel(),
    addressBookViewModel: AddressBookViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    var step by remember { mutableStateOf(ProfileStep.MAIN) }

    LaunchedEffect(Unit) {
        viewModel.refresh()
    }

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

    Column(
        modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(QervonSpacing.md),
        verticalArrangement = Arrangement.spacedBy(QervonSpacing.md),
    ) {
        QervonCard {
            Text("Sadakat Puanı", style = MaterialTheme.typography.labelSmall, color = QervonColors.OnSurfaceMuted)
            Text("${state.profile?.loyaltyPoints ?: 0}", style = MaterialTheme.typography.headlineMedium)
        }

        QervonCard {
            Text("Kayıtlı Adresler", style = MaterialTheme.typography.titleMedium)
            Text(
                "${state.profile?.addresses?.size ?: 0} adres kayıtlı",
                style = MaterialTheme.typography.bodySmall,
                color = QervonColors.OnSurfaceMuted,
                modifier = Modifier.padding(top = QervonSpacing.xs),
            )
            OutlinedButton(
                onClick = { step = ProfileStep.ADDRESS_LIST },
                modifier = Modifier.fillMaxWidth().padding(top = QervonSpacing.sm),
            ) {
                Text("Adresleri Yönet")
            }
        }

        QervonCard {
            Text("Cüzdan/Bakiye", style = MaterialTheme.typography.titleMedium)
            Text(
                "Bu fazda cüzdan özelliği henüz aktif değil.",
                color = QervonColors.OnSurfaceMuted,
                style = MaterialTheme.typography.bodySmall,
                modifier = Modifier.padding(top = QervonSpacing.xs),
            )
        }

        state.errorMessage?.let { Text(it, color = QervonColors.Danger) }

        OutlinedButton(onClick = { viewModel.logout(); onLoggedOut() }, modifier = Modifier.fillMaxWidth()) { Text("Çıkış Yap") }
    }
}
