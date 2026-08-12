// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/ProofOfDeliveryFeature/PickupView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   `POST /v1/courier/orders/{id}/pickup` takes no body and records no
//   proof — the backend does not model pickup evidence today (only
//   delivery evidence). This screen is intentionally a single confirmation
//   step rather than a fake QR/photo capture that would have no effect on
//   the server.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import QervonCore
import QervonNetworking
import QervonDesignSystem

public struct PickupView: View {
    let order: Order
    let api: QervonAPI
    let onPickedUp: (Order) -> Void
    @Environment(\.dismiss) private var dismiss
    @State private var isSubmitting = false
    @State private var errorMessage: String?

    public init(order: Order, api: QervonAPI, onPickedUp: @escaping (Order) -> Void) {
        self.order = order
        self.api = api
        self.onPickedUp = onPickedUp
    }

    public var body: some View {
        VStack(spacing: QervonSpacing.lg) {
            Spacer()
            Image(systemName: "shippingbox.fill")
                .font(.system(size: 48))
                .foregroundColor(QervonColor.accent)
            Text("Teslim Alma")
                .font(.system(size: 20, weight: .bold))
                .foregroundColor(QervonColor.textPrimary)
            Text(order.pickup.label ?? "Alım noktası")
                .font(.system(size: 14))
                .foregroundColor(QervonColor.textSecondary)

            if let errorMessage {
                Text(errorMessage)
                    .font(.system(size: 13))
                    .foregroundColor(QervonColor.danger)
            }

            Button("Paketi Aldım") {
                Task { await confirmPickup() }
            }
            .buttonStyle(QervonButtonStyle(isEnabled: !isSubmitting))
            .disabled(isSubmitting)
            .padding(.horizontal, QervonSpacing.xl)
            Spacer()
        }
        .qervonScreenBackground()
    }

    private func confirmPickup() async {
        isSubmitting = true
        errorMessage = nil
        defer { isSubmitting = false }
        do {
            let updated = try await api.startTransit(orderId: order.id)
            onPickedUp(updated)
            dismiss()
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
