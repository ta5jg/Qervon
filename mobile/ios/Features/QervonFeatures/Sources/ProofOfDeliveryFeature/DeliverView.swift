// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/ProofOfDeliveryFeature/DeliverView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import QervonCore
import QervonNetworking
import QervonDesignSystem

public struct DeliverView: View {
    @StateObject private var viewModel: DeliveryViewModel
    let onDelivered: (Order) -> Void
    @Environment(\.dismiss) private var dismiss

    @State private var showingScanner = false
    @State private var showingSignaturePad = false
    @State private var showingCamera = false

    public init(order: Order, api: QervonAPI, onDelivered: @escaping (Order) -> Void) {
        _viewModel = StateObject(wrappedValue: DeliveryViewModel(order: order, api: api))
        self.onDelivered = onDelivered
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.lg) {
                Text("Teslim Et")
                    .font(.system(size: 20, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
                    .padding(.top, QervonSpacing.lg)

                QervonCard {
                    VStack(spacing: QervonSpacing.md) {
                        QervonTextField(title: "Alıcı Adı", text: $viewModel.recipientName)

                        proofRow(
                            title: "QR / Barkod",
                            isDone: viewModel.qrBarcodeVerified,
                            doneLabel: viewModel.scannedCode ?? "Manuel onaylandı"
                        ) {
                            showingScanner = true
                        }

                        proofRow(
                            title: "İmza",
                            isDone: viewModel.signatureBase64 != nil,
                            doneLabel: "İmza alındı"
                        ) {
                            showingSignaturePad = true
                        }

                        proofRow(
                            title: "Fotoğraf (yalnızca yerel kayıt)",
                            isDone: viewModel.localPhoto != nil,
                            doneLabel: "Fotoğraf çekildi"
                        ) {
                            showingCamera = true
                        }

                        Toggle("QR/Barkod doğrulandı (manuel)", isOn: $viewModel.qrBarcodeVerified)
                            .tint(QervonColor.accent)
                            .foregroundColor(QervonColor.textPrimary)

                        if viewModel.requiresPaymentConfirmation {
                            Toggle("Nakit tahsil edildi", isOn: $viewModel.paymentCollected)
                                .tint(QervonColor.success)
                                .foregroundColor(QervonColor.textPrimary)
                        }

                        if let errorMessage = viewModel.errorMessage {
                            Text(errorMessage)
                                .font(.system(size: 13))
                                .foregroundColor(QervonColor.danger)
                        }

                        Button("Teslimatı Tamamla") {
                            Task {
                                if let delivered = await viewModel.submit() {
                                    onDelivered(delivered)
                                    dismiss()
                                }
                            }
                        }
                        .buttonStyle(QervonButtonStyle(isEnabled: viewModel.canSubmit))
                        .disabled(!viewModel.canSubmit || viewModel.isSubmitting)
                    }
                }
                .padding(.horizontal, QervonSpacing.lg)
            }
            .padding(.bottom, QervonSpacing.xl)
        }
        .qervonScreenBackground()
        .sheet(isPresented: $showingScanner) {
            BarcodeScannerSheet { code in
                viewModel.handleScannedCode(code)
            }
        }
        .sheet(isPresented: $showingSignaturePad) {
            SignaturePadView { base64 in
                viewModel.signatureBase64 = base64
            }
        }
        .sheet(isPresented: $showingCamera) {
            CameraCaptureView { image in
                viewModel.localPhoto = image
            }
        }
    }

    private func proofRow(title: String, isDone: Bool, doneLabel: String, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            HStack {
                Image(systemName: isDone ? "checkmark.circle.fill" : "circle")
                    .foregroundColor(isDone ? QervonColor.success : QervonColor.textSecondary)
                VStack(alignment: .leading) {
                    Text(title)
                        .font(.system(size: 13, weight: .semibold))
                        .foregroundColor(QervonColor.textPrimary)
                    if isDone {
                        Text(doneLabel)
                            .font(.system(size: 11))
                            .foregroundColor(QervonColor.textSecondary)
                    }
                }
                Spacer()
                Image(systemName: "chevron.right")
                    .foregroundColor(QervonColor.textSecondary)
            }
        }
    }
}
