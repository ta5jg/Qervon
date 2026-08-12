// =============================================================================
// File:           mobile/android/feature/proof/src/main/kotlin/com/qervon/features/proof/ProofOfDeliveryViewModel.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Drives `POST /v1/courier/orders/{id}/deliver`, which requires at
//   least one of: a verified QR/barcode scan, a digital signature, or a
//   photo. A captured photo is kept on-device (its local path stays
//   visible to the courier) and also uploaded to
//   `POST /v1/courier/orders/{id}/photo-evidence` before delivery, so
//   `photo_evidence_url` is a real, server-reachable URL rather than a
//   local-only path.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.proof

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.qervon.core.common.QervonApiException
import com.qervon.core.network.QervonApi
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharedFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import javax.inject.Inject

data class ProofUiState(
    val recipientName: String = "",
    val qrBarcodeVerified: Boolean = false,
    val scannedCode: String? = null,
    val signatureBase64: String? = null,
    val localPhotoPath: String? = null,
    val isCashOrder: Boolean = false,
    val paymentCollected: Boolean = false,
    val isSubmitting: Boolean = false,
    val errorMessage: String? = null,
) {
    val canSubmit: Boolean
        get() = recipientName.isNotBlank() && (qrBarcodeVerified || signatureBase64 != null || localPhotoPath != null)
}

sealed class ProofEvent {
    object Delivered : ProofEvent()
}

@HiltViewModel
class ProofOfDeliveryViewModel @Inject constructor(private val api: QervonApi) : ViewModel() {

    private val _uiState = MutableStateFlow(ProofUiState())
    val uiState: StateFlow<ProofUiState> = _uiState.asStateFlow()

    private val _events = MutableSharedFlow<ProofEvent>(extraBufferCapacity = 1)
    val events: SharedFlow<ProofEvent> = _events.asSharedFlow()

    fun setCashOrder(isCash: Boolean) {
        _uiState.value = _uiState.value.copy(isCashOrder = isCash)
    }

    fun onRecipientNameChanged(value: String) {
        _uiState.value = _uiState.value.copy(recipientName = value)
    }

    fun onBarcodeScanned(code: String) {
        _uiState.value = _uiState.value.copy(qrBarcodeVerified = true, scannedCode = code)
    }

    fun onSignatureCaptured(base64Png: String?) {
        _uiState.value = _uiState.value.copy(signatureBase64 = base64Png)
    }

    fun onPhotoCaptured(localPath: String?) {
        _uiState.value = _uiState.value.copy(localPhotoPath = localPath)
    }

    fun onPaymentCollectedChanged(value: Boolean) {
        _uiState.value = _uiState.value.copy(paymentCollected = value)
    }

    fun submit(orderId: String) {
        val state = _uiState.value
        if (!state.canSubmit) {
            _uiState.value = state.copy(errorMessage = "Alıcı adı ve en az bir teslim kanıtı (QR, imza veya fotoğraf) gerekli.")
            return
        }
        viewModelScope.launch {
            _uiState.value = state.copy(isSubmitting = true, errorMessage = null)
            try {
                // A failed upload should not block delivery when a
                // QR/signature proof already satisfies `canSubmit` — the
                // photo stays available locally either way.
                val photoEvidenceUrl = state.localPhotoPath?.let { path ->
                    try {
                        val jpegBytes = java.io.File(path).readBytes()
                        api.uploadDeliveryPhoto(orderId, jpegBytes)
                    } catch (error: Exception) {
                        null
                    }
                }
                api.deliverOrder(
                    orderId = orderId,
                    recipientName = state.recipientName.trim(),
                    qrBarcodeVerified = state.qrBarcodeVerified,
                    digitalSignatureBase64 = state.signatureBase64,
                    photoEvidenceUrl = photoEvidenceUrl,
                    paymentCollected = state.paymentCollected,
                )
                _events.tryEmit(ProofEvent.Delivered)
            } catch (error: QervonApiException) {
                _uiState.value = _uiState.value.copy(errorMessage = error.message)
            } finally {
                _uiState.value = _uiState.value.copy(isSubmitting = false)
            }
        }
    }
}
