// =============================================================================
// File:           mobile/android/feature/earnings/src/main/kotlin/com/qervon/features/earnings/EarningsViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Loads the courier's wallet and ratings. The "today / this week /
//   this month" earnings split is computed entirely client-side from the
//   transaction list — the backend has no period-aggregation endpoint
//   (see `CourierWallet.totalCreditedSince` in core:common).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.earnings

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.qervon.core.common.QervonApiException
import com.qervon.core.common.QervonFormat
import com.qervon.core.common.model.CourierWallet
import com.qervon.core.common.model.CustomerRating
import com.qervon.core.common.model.Money
import com.qervon.core.common.model.averageStars
import com.qervon.core.network.QervonApi
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

data class EarningsUiState(
    val wallet: CourierWallet? = null,
    val ratings: List<CustomerRating> = emptyList(),
    val isLoading: Boolean = false,
    val errorMessage: String? = null,
) {
    val averageRating: Double? get() = ratings.averageStars()
    val todayEarnings: Money? get() = wallet?.totalCreditedSince(QervonFormat.startOfDay())
    val weekEarnings: Money? get() = wallet?.totalCreditedSince(QervonFormat.startOfWeek())
    val monthEarnings: Money? get() = wallet?.totalCreditedSince(QervonFormat.startOfMonth())
}

@HiltViewModel
class EarningsViewModel @Inject constructor(private val api: QervonApi) : ViewModel() {

    private val _uiState = MutableStateFlow(EarningsUiState())
    val uiState: StateFlow<EarningsUiState> = _uiState.asStateFlow()

    fun refresh() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true, errorMessage = null)
            try {
                val wallet = api.getOwnWallet()
                val ratings = api.getOwnRatings()
                _uiState.value = _uiState.value.copy(wallet = wallet, ratings = ratings)
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isLoading = false)
            }
        }
    }
}
