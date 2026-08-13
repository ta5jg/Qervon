// =============================================================================
// File:           mobile/android/feature/customerorder/src/main/kotlin/com/qervon/features/customerorder/NewOrderScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.customerorder

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
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Surface
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
import com.qervon.core.common.model.Address
import com.qervon.core.common.model.PaymentMethod
import com.qervon.core.designsystem.QervonCard
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing
import com.qervon.features.addressbook.MapAddressPickerScreen
import com.qervon.features.addressbook.PickedLocation

private enum class NewOrderStep { FORM, PICK_PICKUP, PICK_DROPOFF }

private val paymentOptions = listOf(
    PaymentMethod.CASH to "Nakit",
    PaymentMethod.CARD to "Kart",
    PaymentMethod.QR to "QR",
    PaymentMethod.WALLET to "Cüzdan",
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NewOrderScreen(
    onOrderCreated: (orderId: String) -> Unit,
    viewModel: NewOrderViewModel = hiltViewModel(),
) {
    val state by viewModel.uiState.collectAsStateWithLifecycle()
    var step by remember { mutableStateOf(NewOrderStep.FORM) }

    LaunchedEffect(Unit) {
        viewModel.events.collect { event ->
            if (event is NewOrderEvent.Created) onOrderCreated(event.order.id)
        }
    }

    when (step) {
        NewOrderStep.PICK_PICKUP -> MapAddressPickerScreen(
            onPicked = { picked: PickedLocation ->
                viewModel.setPickup(Address(picked.latitude, picked.longitude, picked.address))
                step = NewOrderStep.FORM
            },
            onClose = { step = NewOrderStep.FORM },
        )
        NewOrderStep.PICK_DROPOFF -> MapAddressPickerScreen(
            onPicked = { picked: PickedLocation ->
                viewModel.setDropoff(Address(picked.latitude, picked.longitude, picked.address))
                step = NewOrderStep.FORM
            },
            onClose = { step = NewOrderStep.FORM },
        )
        NewOrderStep.FORM -> Column(
            modifier = Modifier.fillMaxSize().verticalScroll(rememberScrollState()).padding(QervonSpacing.md),
            verticalArrangement = Arrangement.spacedBy(QervonSpacing.md),
        ) {
            AddressPickerRow("Alım Noktası", state.pickup?.label, onClick = { step = NewOrderStep.PICK_PICKUP })
            AddressPickerRow("Teslim Noktası", state.dropoff?.label, onClick = { step = NewOrderStep.PICK_DROPOFF })

            if (state.fareQuote != null || state.isQuoting) {
                QervonCard {
                    Text("Tahmini Ücret", style = MaterialTheme.typography.titleMedium)
                    Text(
                        if (state.isQuoting) "Hesaplanıyor…" else state.fareQuote?.money?.formatted().orEmpty(),
                        style = MaterialTheme.typography.headlineMedium,
                    )
                    state.fareQuote?.let {
                        Text(String.format("%.1f km", it.distanceKm), color = QervonColors.OnSurfaceMuted)
                    }
                }
            }

            Text("Ödeme Yöntemi", style = MaterialTheme.typography.titleMedium)
            SingleChoiceSegmentedButtonRow(modifier = Modifier.fillMaxWidth()) {
                paymentOptions.forEachIndexed { index, (method, label) ->
                    SegmentedButton(
                        selected = state.paymentMethod == method.name.lowercase(),
                        onClick = { viewModel.onPaymentMethodChanged(method.name.lowercase()) },
                        shape = SegmentedButtonDefaults.itemShape(index, paymentOptions.size),
                    ) { Text(label) }
                }
            }

            OutlinedTextField(
                value = state.couponCode,
                onValueChange = viewModel::onCouponCodeChanged,
                label = { Text("Kupon kodu (opsiyonel)") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = state.deliveryNote,
                onValueChange = viewModel::onDeliveryNoteChanged,
                label = { Text("Teslimat notu (opsiyonel)") },
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = state.contactPhone,
                onValueChange = viewModel::onContactPhoneChanged,
                label = { Text("İletişim telefonu (opsiyonel)") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )

            state.errorMessage?.let {
                Surface(color = QervonColors.Danger.copy(alpha = 0.1f)) {
                    Text(it, color = QervonColors.Danger, modifier = Modifier.padding(QervonSpacing.sm))
                }
            }

            QervonPrimaryButton(
                text = "Sipariş Ver",
                onClick = viewModel::submit,
                enabled = state.canSubmit,
                isLoading = state.isSubmitting,
            )
        }
    }
}

@Composable
private fun AddressPickerRow(label: String, value: String?, onClick: () -> Unit) {
    QervonCard {
        Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Column {
                Text(label, style = MaterialTheme.typography.labelSmall, color = QervonColors.OnSurfaceMuted)
                Text(value ?: "Seçilmedi", style = MaterialTheme.typography.bodyMedium)
            }
            OutlinedButton(onClick = onClick) { Text(if (value == null) "Seç" else "Değiştir") }
        }
    }
}
