// =============================================================================
// File:           mobile/android/feature/earnings/src/main/kotlin/com/qervon/features/earnings/EarningsScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.earnings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.ExperimentalMaterial3Api
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
import com.qervon.core.common.QervonFormat
import com.qervon.core.common.model.WalletTransaction
import com.qervon.core.designsystem.QervonCard
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonSpacing

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun EarningsScreen(viewModel: EarningsViewModel = hiltViewModel()) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(Unit) { viewModel.refresh() }

    Scaffold(topBar = { TopAppBar(title = { Text("Kazançlarım") }) }) { padding ->
        LazyColumn(
            modifier = Modifier.fillMaxSize().padding(padding),
            contentPadding = androidx.compose.foundation.layout.PaddingValues(QervonSpacing.md),
            verticalArrangement = Arrangement.spacedBy(QervonSpacing.md),
        ) {
            item {
                QervonCard {
                    Text("Bakiye", style = MaterialTheme.typography.bodyMedium, color = QervonColors.OnSurfaceMuted)
                    Text(state.wallet?.balance?.formatted() ?: "—", style = MaterialTheme.typography.headlineMedium)
                    state.averageRating?.let {
                        Text(String.format("%.1f ★ ortalama puan", it), color = QervonColors.Warning)
                    }
                }
            }
            item {
                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(QervonSpacing.sm)) {
                    EarningsPeriodCard("Bugün", state.todayEarnings?.formatted())
                    EarningsPeriodCard("Bu Hafta", state.weekEarnings?.formatted())
                    EarningsPeriodCard("Bu Ay", state.monthEarnings?.formatted())
                }
            }
            item { Text("İşlem Geçmişi", style = MaterialTheme.typography.titleMedium) }
            items(state.wallet?.transactions.orEmpty(), key = { it.id }) { transaction ->
                TransactionRow(transaction)
            }
        }
    }
}

@Composable
private fun EarningsPeriodCard(label: String, amount: String?) {
    QervonCard {
        Text(label, style = MaterialTheme.typography.labelSmall, color = QervonColors.OnSurfaceMuted)
        Text(amount ?: "—", style = MaterialTheme.typography.titleMedium)
    }
}

@Composable
private fun TransactionRow(transaction: WalletTransaction) {
    QervonCard {
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Column {
                Text(transaction.transactionType.displayName(), style = MaterialTheme.typography.bodyMedium)
                Text(QervonFormat.dayAndTime(transaction.createdAt), style = MaterialTheme.typography.labelSmall, color = QervonColors.OnSurfaceMuted)
            }
            Text(
                transaction.money.formatted(),
                color = if (transaction.transactionType.isCredit) QervonColors.Success else QervonColors.Danger,
            )
        }
    }
}
