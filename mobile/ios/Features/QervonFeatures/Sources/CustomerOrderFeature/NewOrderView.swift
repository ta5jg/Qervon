// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/CustomerOrderFeature/NewOrderView.swift
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
import AddressBookFeature

public struct NewOrderView: View {
    @StateObject private var viewModel: NewOrderViewModel
    private let api: QervonAPI
    let onOrderCreated: (Order) -> Void

    @State private var pickingPickup = false
    @State private var pickingDropoff = false

    public init(api: QervonAPI, onOrderCreated: @escaping (Order) -> Void) {
        self.api = api
        self.onOrderCreated = onOrderCreated
        _viewModel = StateObject(wrappedValue: NewOrderViewModel(api: api))
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.lg) {
                Text("Yeni Sipariş")
                    .font(.system(size: 20, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
                    .padding(.top, QervonSpacing.lg)

                QervonCard {
                    VStack(spacing: QervonSpacing.md) {
                        addressRow(title: "Alım Adresi", address: viewModel.pickup) {
                            pickingPickup = true
                        }
                        addressRow(title: "Teslim Adresi", address: viewModel.dropoff) {
                            pickingDropoff = true
                        }
                    }
                }
                .padding(.horizontal, QervonSpacing.lg)

                if viewModel.isQuoting {
                    ProgressView("Ücret hesaplanıyor…").tint(QervonColor.accent)
                } else if let quote = viewModel.quote {
                    QervonCard(accentBorder: QervonColor.success) {
                        VStack(spacing: QervonSpacing.xs) {
                            Text("TAHMİNİ ÜCRET")
                                .font(.system(size: 11, weight: .bold))
                                .foregroundColor(QervonColor.textSecondary)
                            Text(quote.money.formatted)
                                .font(.system(size: 24, weight: .bold))
                                .foregroundColor(QervonColor.success)
                            Text(String(format: "%.1f km", quote.distanceKm))
                                .font(.system(size: 12))
                                .foregroundColor(QervonColor.textSecondary)
                            Text("Nihai ücret sipariş oluşturulduğunda sunucu tarafında kesinleşir.")
                                .font(.system(size: 10))
                                .foregroundColor(QervonColor.textSecondary)
                                .multilineTextAlignment(.center)
                        }
                        .frame(maxWidth: .infinity)
                    }
                    .padding(.horizontal, QervonSpacing.lg)
                }

                QervonCard {
                    VStack(spacing: QervonSpacing.md) {
                        Picker("Ödeme Yöntemi", selection: $viewModel.paymentMethod) {
                            ForEach(PaymentMethod.allCases, id: \.self) { method in
                                Text(method.displayName).tag(method)
                            }
                        }
                        .pickerStyle(.segmented)

                        QervonTextField(title: "Kupon Kodu (opsiyonel)", text: $viewModel.couponCode, autocapitalize: false)
                        QervonTextField(title: "Teslimat Notu (opsiyonel)", text: $viewModel.deliveryNote)
                        QervonTextField(
                            title: "İletişim Telefonu (opsiyonel)", text: $viewModel.contactPhone, keyboard: .phonePad
                        )

                        if let errorMessage = viewModel.errorMessage {
                            Text(errorMessage)
                                .font(.system(size: 13))
                                .foregroundColor(QervonColor.danger)
                        }

                        Button("Sipariş Oluştur") {
                            Task {
                                if let order = await viewModel.submit() {
                                    onOrderCreated(order)
                                    viewModel.reset()
                                }
                            }
                        }
                        .buttonStyle(QervonButtonStyle(isEnabled: viewModel.canSubmit))
                        .disabled(!viewModel.canSubmit)
                    }
                }
                .padding(.horizontal, QervonSpacing.lg)
            }
            .padding(.bottom, QervonSpacing.xl)
        }
        .qervonScreenBackground()
        .sheet(isPresented: $pickingPickup) {
            AddressPickerSheet(title: "Alım Adresini Seç", api: api) { address in
                viewModel.setPickup(address)
            }
        }
        .sheet(isPresented: $pickingDropoff) {
            AddressPickerSheet(title: "Teslim Adresini Seç", api: api) { address in
                viewModel.setDropoff(address)
            }
        }
    }

    private func addressRow(title: String, address: Address?, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text(title)
                        .font(.system(size: 11, weight: .semibold))
                        .foregroundColor(QervonColor.textSecondary)
                    Text(address?.label ?? "Seçilmedi")
                        .font(.system(size: 14, weight: .bold))
                        .foregroundColor(address == nil ? QervonColor.textSecondary : QervonColor.textPrimary)
                }
                Spacer()
                Image(systemName: "chevron.right")
                    .foregroundColor(QervonColor.textSecondary)
            }
        }
    }
}
