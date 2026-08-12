// =============================================================================
// File:           mobile/android/feature/addressbook/src/main/kotlin/com/qervon/features/addressbook/AddressBookViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Loads/mutates the customer's saved addresses
//   (`POST/DELETE /v1/customer/profile/addresses`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.addressbook

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.qervon.core.common.QervonApiException
import com.qervon.core.common.model.SavedAddress
import com.qervon.core.network.QervonApi
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

data class AddressBookUiState(
    val addresses: List<SavedAddress> = emptyList(),
    val isLoading: Boolean = false,
    val errorMessage: String? = null,
)

@HiltViewModel
class AddressBookViewModel @Inject constructor(private val api: QervonApi) : ViewModel() {

    private val _uiState = MutableStateFlow(AddressBookUiState())
    val uiState: StateFlow<AddressBookUiState> = _uiState.asStateFlow()

    fun refresh() {
        viewModelScope.launch {
            _uiState.value = _uiState.value.copy(isLoading = true, errorMessage = null)
            try {
                val profile = api.getCustomerProfile()
                _uiState.value = _uiState.value.copy(addresses = profile.addresses)
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isLoading = false)
            }
        }
    }

    fun addAddress(label: String, latitude: Double, longitude: Double, fullAddress: String) {
        viewModelScope.launch {
            try {
                val profile = api.addAddress(label, latitude, longitude, fullAddress)
                _uiState.value = _uiState.value.copy(addresses = profile.addresses)
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            }
        }
    }

    fun removeAddress(addressId: String) {
        viewModelScope.launch {
            try {
                val profile = api.removeAddress(addressId)
                _uiState.value = _uiState.value.copy(addresses = profile.addresses)
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            }
        }
    }
}
