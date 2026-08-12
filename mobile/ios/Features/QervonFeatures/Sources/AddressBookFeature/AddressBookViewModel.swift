// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/AddressBookFeature/AddressBookViewModel.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Loads and mutates the customer's saved address book
//   (`GET/POST/DELETE /v1/customer/profile/addresses`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import QervonCore
import QervonNetworking

@MainActor
public final class AddressBookViewModel: ObservableObject {
    @Published public private(set) var addresses: [SavedAddress] = []
    @Published public private(set) var isLoading = false
    @Published public var errorMessage: String?

    private let api: QervonAPI

    public init(api: QervonAPI) {
        self.api = api
    }

    public func load() async {
        isLoading = true
        errorMessage = nil
        defer { isLoading = false }
        do {
            let profile = try await api.getCustomerProfile()
            addresses = profile.addresses
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    public func addAddress(label: String, coordinate: GeoLocation, fullAddress: String) async {
        errorMessage = nil
        do {
            let profile = try await api.addAddress(
                label: label,
                latitude: coordinate.latitude,
                longitude: coordinate.longitude,
                fullAddress: fullAddress
            )
            addresses = profile.addresses
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    public func removeAddress(_ address: SavedAddress) async {
        errorMessage = nil
        do {
            let profile = try await api.removeAddress(id: address.id)
            addresses = profile.addresses
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
