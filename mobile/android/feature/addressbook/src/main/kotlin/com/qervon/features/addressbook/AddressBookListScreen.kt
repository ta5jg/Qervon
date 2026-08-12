// =============================================================================
// File:           mobile/android/feature/addressbook/src/main/kotlin/com/qervon/features/addressbook/AddressBookListScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.addressbook

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.qervon.core.designsystem.QervonCard
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonSpacing

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AddressBookListScreen(
    onAddAddress: () -> Unit,
    viewModel: AddressBookViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(Unit) { viewModel.refresh() }

    Scaffold(
        topBar = { TopAppBar(title = { Text("Adres Defterim") }) },
        floatingActionButton = {
            FloatingActionButton(onClick = onAddAddress) { Icon(Icons.Filled.Add, contentDescription = "Ekle") }
        },
    ) { padding ->
        if (state.addresses.isEmpty()) {
            Column(modifier = Modifier.fillMaxSize().padding(padding).padding(QervonSpacing.lg)) {
                Text("Henüz kayıtlı adresiniz yok.", color = QervonColors.OnSurfaceMuted)
            }
            return@Scaffold
        }
        LazyColumn(
            modifier = Modifier.fillMaxSize().padding(padding),
            contentPadding = androidx.compose.foundation.layout.PaddingValues(QervonSpacing.md),
            verticalArrangement = Arrangement.spacedBy(QervonSpacing.sm),
        ) {
            items(state.addresses, key = { it.id }) { address ->
                QervonCard {
                    Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                        Column {
                            Text(address.label, style = MaterialTheme.typography.titleMedium)
                            Text(address.fullAddress, color = QervonColors.OnSurfaceMuted)
                        }
                        IconButton(onClick = { viewModel.removeAddress(address.id) }) {
                            Icon(Icons.Filled.Delete, contentDescription = "Sil")
                        }
                    }
                }
            }
        }
    }
}
