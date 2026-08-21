// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/ProofOfDeliveryFeature/PickupView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Captures and uploads the mandatory pickup photo before transitioning an
//   assigned courier order to in-transit.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import UIKit
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
    @State private var pickupPhoto: UIImage?
    @State private var showingCamera = false

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

            Button(pickupPhoto == nil ? "Teslim Alma Fotoğrafı Çek" : "Fotoğrafı Yeniden Çek") {
                showingCamera = true
            }
            .buttonStyle(QervonButtonStyle(kind: .secondary))
            .padding(.horizontal, QervonSpacing.xl)

            if pickupPhoto != nil {
                Label("Fotoğraf hazır", systemImage: "checkmark.circle.fill")
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundColor(QervonColor.success)
            }

            if let errorMessage {
                Text(errorMessage)
                    .font(.system(size: 13))
                    .foregroundColor(QervonColor.danger)
            }

            Button("Fotoğrafı Yükle ve Paketi Al") {
                Task { await confirmPickup() }
            }
            .buttonStyle(QervonButtonStyle(isEnabled: pickupPhoto != nil && !isSubmitting))
            .disabled(pickupPhoto == nil || isSubmitting)
            .padding(.horizontal, QervonSpacing.xl)
            Spacer()
        }
        .qervonScreenBackground()
        .sheet(isPresented: $showingCamera) {
            CameraCaptureView { image in
                pickupPhoto = image
            }
        }
    }

    private func confirmPickup() async {
        isSubmitting = true
        errorMessage = nil
        defer { isSubmitting = false }
        guard let pickupPhoto, let jpegData = pickupPhoto.jpegData(compressionQuality: 0.7) else {
            errorMessage = "Teslim alma fotoğrafı zorunludur."
            return
        }
        do {
            _ = LocalDeliveryEvidenceStore.save(pickupPhoto, forOrderId: "pickup-\(order.id.uuidString)")
            let evidenceURL = try await api.uploadOrderEvidencePhoto(orderId: order.id, jpegData: jpegData)
            let updated = try await api.startTransit(
                orderId: order.id,
                pickupPhotoEvidenceURL: evidenceURL
            )
            onPickedUp(updated)
            dismiss()
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
