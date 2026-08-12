// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/AddressBookFeature/AddressBookListView.swift
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

/// Standalone address-book management screen (embedded in Profile).
public struct AddressBookListView: View {
    @StateObject private var viewModel: AddressBookViewModel
    @State private var showingMapPicker = false

    public init(api: QervonAPI) {
        _viewModel = StateObject(wrappedValue: AddressBookViewModel(api: api))
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.md) {
                if viewModel.addresses.isEmpty && !viewModel.isLoading {
                    QervonCard {
                        Text("Henüz kayıtlı adresiniz yok.")
                            .font(.system(size: 13))
                            .foregroundColor(QervonColor.textSecondary)
                    }
                    .padding(.horizontal, QervonSpacing.lg)
                } else {
                    ForEach(viewModel.addresses) { address in
                        addressRow(address)
                            .padding(.horizontal, QervonSpacing.lg)
                    }
                }

                if let errorMessage = viewModel.errorMessage {
                    Text(errorMessage)
                        .font(.system(size: 13))
                        .foregroundColor(QervonColor.danger)
                        .padding(.horizontal, QervonSpacing.lg)
                }

                Button("Yeni Adres Ekle") {
                    showingMapPicker = true
                }
                .buttonStyle(QervonButtonStyle(kind: .secondary))
                .padding(.horizontal, QervonSpacing.lg)
            }
            .padding(.vertical, QervonSpacing.lg)
        }
        .qervonScreenBackground()
        .navigationTitle("Adres Defteri")
        .task { await viewModel.load() }
        .sheet(isPresented: $showingMapPicker) {
            MapAddressPickerView { coordinate, fullAddress in
                Task {
                    let label = fullAddress.components(separatedBy: ",").first ?? "Adres"
                    await viewModel.addAddress(label: label, coordinate: coordinate, fullAddress: fullAddress)
                }
            }
        }
    }

    private func addressRow(_ address: SavedAddress) -> some View {
        QervonCard {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    HStack(spacing: QervonSpacing.xs) {
                        Text(address.label)
                            .font(.system(size: 14, weight: .bold))
                            .foregroundColor(QervonColor.textPrimary)
                        if address.isDefault {
                            Text("VARSAYILAN")
                                .font(.system(size: 9, weight: .bold))
                                .foregroundColor(QervonColor.accent)
                        }
                    }
                    Text(address.fullAddress)
                        .font(.system(size: 12))
                        .foregroundColor(QervonColor.textSecondary)
                }
                Spacer()
                Button {
                    Task { await viewModel.removeAddress(address) }
                } label: {
                    Image(systemName: "trash")
                        .foregroundColor(QervonColor.danger)
                }
            }
        }
    }
}
