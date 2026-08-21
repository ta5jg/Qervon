// =============================================================================
// File:           mobile/android/feature/proof/src/main/kotlin/com/qervon/features/proof/ProofOfDeliveryScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Container screen for the delivery-confirmation flow: recipient name,
//   one of {QR/barcode, signature, photo} as evidence, and — for
//   cash-method orders — a "cash collected" confirmation, then submits
//   to `POST /v1/courier/orders/{id}/deliver`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.proof

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Checkbox
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.qervon.core.designsystem.QervonCard
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing

private enum class ProofStep { FORM, SCANNING, SIGNING, PHOTOGRAPHING }

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun ProofOfDeliveryScreen(
    orderId: String,
    isCashOrder: Boolean,
    onDelivered: () -> Unit,
    onClose: () -> Unit,
    viewModel: ProofOfDeliveryViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    var step by rememberSaveable { mutableStateOf(ProofStep.FORM) }

    LaunchedEffect(Unit) {
        viewModel.setCashOrder(isCashOrder)
        viewModel.events.collect { onDelivered() }
    }

    when (step) {
        ProofStep.SCANNING -> BarcodeScannerScreen(
            onCodeDetected = { code -> viewModel.onBarcodeScanned(code); step = ProofStep.FORM },
            onClose = { step = ProofStep.FORM },
        )
        ProofStep.SIGNING -> SignatureCaptureScreen(
            onCaptured = { base64 -> viewModel.onSignatureCaptured(base64); step = ProofStep.FORM },
            onClose = { step = ProofStep.FORM },
        )
        ProofStep.PHOTOGRAPHING -> PhotoCaptureScreen(
            onCaptured = { path -> viewModel.onPhotoCaptured(path); step = ProofStep.FORM },
            onClose = { step = ProofStep.FORM },
        )
        ProofStep.FORM -> Scaffold(topBar = { TopAppBar(title = { Text("Teslimatı Onayla") }) }) { padding ->
            Column(
                modifier = Modifier.fillMaxSize().padding(padding).padding(QervonSpacing.md),
                verticalArrangement = Arrangement.spacedBy(QervonSpacing.md),
            ) {
                OutlinedTextField(
                    value = state.recipientName,
                    onValueChange = viewModel::onRecipientNameChanged,
                    label = { Text("Alıcının adı") },
                    singleLine = true,
                    modifier = Modifier.fillMaxWidth(),
                )

                QervonCard {
                    Text("Teslim Kanıtı (en az biri gerekli)", style = MaterialTheme.typography.titleMedium)
                    EvidenceRow("QR / Barkod", state.qrBarcodeVerified, onClick = { step = ProofStep.SCANNING })
                    EvidenceRow("İmza", state.signatureBase64 != null, onClick = { step = ProofStep.SIGNING })
                    EvidenceRow("Fotoğraf (sunucuya güvenli yüklenir)", state.localPhotoPath != null, onClick = { step = ProofStep.PHOTOGRAPHING })
                }

                if (state.isCashOrder) {
                    QervonCard {
                        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                            Checkbox(checked = state.paymentCollected, onCheckedChange = viewModel::onPaymentCollectedChanged)
                            Text("Nakit ücret tahsil edildi")
                        }
                    }
                }

                state.errorMessage?.let {
                    Surface(color = QervonColors.Danger.copy(alpha = 0.1f)) {
                        Text(it, color = QervonColors.Danger, modifier = Modifier.padding(QervonSpacing.sm))
                    }
                }

                QervonPrimaryButton(
                    text = "Teslimatı Tamamla",
                    onClick = { viewModel.submit(orderId) },
                    enabled = state.canSubmit,
                    isLoading = state.isSubmitting,
                )
                OutlinedButton(onClick = onClose, modifier = Modifier.fillMaxWidth()) { Text("Vazgeç") }
            }
        }
    }
}

@Composable
private fun EvidenceRow(label: String, isCaptured: Boolean, onClick: () -> Unit) {
    Row(
        modifier = Modifier.fillMaxWidth().padding(vertical = QervonSpacing.xs),
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Text(label)
        OutlinedButton(onClick = onClick) { Text(if (isCaptured) "Yeniden al" else "Al") }
    }
}
