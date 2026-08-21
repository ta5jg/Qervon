// =============================================================================
// File:           mobile/android/feature/proof/src/main/kotlin/com/qervon/features/proof/PickupProofViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-21
// Version:        0.1.0
//
// Description:
//   Uploads required pickup photo evidence before transitioning an assigned
//   courier order to in-transit. A failed upload never advances the order.
// =============================================================================

package com.qervon.features.proof

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.qervon.core.network.QervonApi
import dagger.hilt.android.lifecycle.HiltViewModel
import java.io.File
import javax.inject.Inject
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch

data class PickupProofUiState(
    val localPhotoPath: String? = null,
    val isSubmitting: Boolean = false,
    val errorMessage: String? = null,
) {
    val canSubmit: Boolean get() = !localPhotoPath.isNullOrBlank() && !isSubmitting
}

sealed class PickupProofEvent {
    object PickedUp : PickupProofEvent()
}

@HiltViewModel
class PickupProofViewModel @Inject constructor(private val api: QervonApi) : ViewModel() {
    private val _uiState = MutableStateFlow(PickupProofUiState())
    val uiState: StateFlow<PickupProofUiState> = _uiState.asStateFlow()

    private val _events = MutableSharedFlow<PickupProofEvent>(extraBufferCapacity = 1)
    val events: SharedFlow<PickupProofEvent> = _events.asSharedFlow()

    fun onPhotoCaptured(localPath: String) {
        _uiState.value = PickupProofUiState(localPhotoPath = localPath)
    }

    fun retakePhoto() {
        _uiState.value = PickupProofUiState()
    }

    fun submit(orderId: String) {
        val current = _uiState.value
        val localPath = current.localPhotoPath
        if (localPath.isNullOrBlank()) {
            _uiState.value = current.copy(errorMessage = "Teslim alma fotoğrafı zorunludur.")
            return
        }
        viewModelScope.launch {
            _uiState.value = current.copy(isSubmitting = true, errorMessage = null)
            try {
                val jpegBytes = File(localPath).readBytes()
                val evidenceUrl = api.uploadOrderEvidencePhoto(orderId, jpegBytes)
                api.pickupOrder(orderId, evidenceUrl)
                _events.tryEmit(PickupProofEvent.PickedUp)
            } catch (error: Exception) {
                _uiState.value = _uiState.value.copy(
                    errorMessage = error.message ?: "Teslim alma kanıtı gönderilemedi.",
                )
            } finally {
                _uiState.value = _uiState.value.copy(isSubmitting = false)
            }
        }
    }
}
