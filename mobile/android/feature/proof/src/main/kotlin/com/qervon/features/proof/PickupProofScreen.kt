// =============================================================================
// File:           mobile/android/feature/proof/src/main/kotlin/com/qervon/features/proof/PickupProofScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-21
// Version:        0.1.0
//
// Description:
//   Mandatory camera evidence flow used before a courier can start transit.
// =============================================================================

package com.qervon.features.proof

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing

@Composable
fun PickupProofScreen(
    orderId: String,
    onPickedUp: () -> Unit,
    onClose: () -> Unit,
    viewModel: PickupProofViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()

    LaunchedEffect(Unit) {
        viewModel.events.collect { event ->
            if (event is PickupProofEvent.PickedUp) onPickedUp()
        }
    }

    if (state.localPhotoPath == null) {
        PhotoCaptureScreen(
            title = "Teslim Alma Fotoğrafı",
            onCaptured = viewModel::onPhotoCaptured,
            onClose = onClose,
        )
        return
    }

    Column(
        modifier = Modifier.fillMaxSize().padding(QervonSpacing.lg),
        verticalArrangement = Arrangement.spacedBy(QervonSpacing.md),
    ) {
        Text("Teslim alma fotoğrafı çekildi.")
        Text(
            "Fotoğraf sunucuya yüklendikten sonra sipariş yolda durumuna geçirilecek.",
            color = QervonColors.OnSurfaceMuted,
        )

        state.errorMessage?.let { message ->
            Surface(color = QervonColors.Danger.copy(alpha = 0.1f)) {
                Text(message, color = QervonColors.Danger, modifier = Modifier.padding(QervonSpacing.sm))
            }
        }

        QervonPrimaryButton(
            text = "Fotoğrafı Yükle ve Teslim Al",
            onClick = { viewModel.submit(orderId) },
            enabled = state.canSubmit,
            isLoading = state.isSubmitting,
        )
        OutlinedButton(
            onClick = viewModel::retakePhoto,
            enabled = !state.isSubmitting,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("Fotoğrafı Yeniden Çek")
        }
        OutlinedButton(
            onClick = onClose,
            enabled = !state.isSubmitting,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("Vazgeç")
        }
    }
}
