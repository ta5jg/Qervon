// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/AddressBookFeature/AddressPickerSheet.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Lets the customer choose a pickup/dropoff point either from their
//   saved address book or by dropping a fresh pin on the map.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import QervonCore
import QervonNetworking
import QervonDesignSystem

public struct AddressPickerSheet: View {
    let title: String
    let api: QervonAPI
    let onPicked: (Address) -> Void

    @StateObject private var viewModel: AddressBookViewModel
    @State private var showingMapPicker = false
    @Environment(\.dismiss) private var dismiss

    public init(title: String, api: QervonAPI, onPicked: @escaping (Address) -> Void) {
        self.title = title
        self.api = api
        self.onPicked = onPicked
        _viewModel = StateObject(wrappedValue: AddressBookViewModel(api: api))
    }

    public var body: some View {
        VStack(spacing: QervonSpacing.md) {
            Text(title)
                .font(.system(size: 16, weight: .bold))
                .foregroundColor(QervonColor.textPrimary)
                .padding(.top, QervonSpacing.lg)

            Button("Haritada Yeni Konum Seç") {
                showingMapPicker = true
            }
            .buttonStyle(QervonButtonStyle(kind: .secondary))
            .padding(.horizontal, QervonSpacing.lg)

            if viewModel.addresses.isEmpty {
                Text("Kayıtlı adresiniz yok.")
                    .font(.system(size: 13))
                    .foregroundColor(QervonColor.textSecondary)
                Spacer()
            } else {
                ScrollView {
                    VStack(spacing: QervonSpacing.sm) {
                        ForEach(viewModel.addresses) { address in
                            Button {
                                onPicked(Address(
                                    latitude: address.location.latitude,
                                    longitude: address.location.longitude,
                                    label: address.label
                                ))
                                dismiss()
                            } label: {
                                QervonCard {
                                    VStack(alignment: .leading, spacing: 2) {
                                        Text(address.label)
                                            .font(.system(size: 14, weight: .bold))
                                            .foregroundColor(QervonColor.textPrimary)
                                        Text(address.fullAddress)
                                            .font(.system(size: 12))
                                            .foregroundColor(QervonColor.textSecondary)
                                    }
                                    .frame(maxWidth: .infinity, alignment: .leading)
                                }
                            }
                        }
                    }
                    .padding(.horizontal, QervonSpacing.lg)
                }
            }
        }
        .qervonScreenBackground()
        .task { await viewModel.load() }
        .sheet(isPresented: $showingMapPicker) {
            MapAddressPickerView { coordinate, fullAddress in
                onPicked(Address(latitude: coordinate.latitude, longitude: coordinate.longitude, label: fullAddress))
                dismiss()
            }
        }
    }
}
