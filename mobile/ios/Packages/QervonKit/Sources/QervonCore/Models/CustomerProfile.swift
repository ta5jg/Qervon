// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/CustomerProfile.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `qervon_domain::{CustomerProfile, SavedAddress}` as returned by
//   `GET /v1/customer/profile` and the address book endpoints.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public struct SavedAddress: Codable, Sendable, Equatable, Identifiable {
    public let id: UUID
    public let label: String
    public let location: GeoLocation
    public let fullAddress: String
    public let isDefault: Bool

    public init(id: UUID, label: String, location: GeoLocation, fullAddress: String, isDefault: Bool) {
        self.id = id
        self.label = label
        self.location = location
        self.fullAddress = fullAddress
        self.isDefault = isDefault
    }
}

public struct CustomerProfile: Codable, Sendable, Equatable {
    public let id: UUID
    public let userId: UUID
    public let companyName: String?
    public let taxId: String?
    public let addresses: [SavedAddress]
    public let loyaltyPoints: Int
    public let createdAt: Date

    public init(
        id: UUID,
        userId: UUID,
        companyName: String?,
        taxId: String?,
        addresses: [SavedAddress],
        loyaltyPoints: Int,
        createdAt: Date
    ) {
        self.id = id
        self.userId = userId
        self.companyName = companyName
        self.taxId = taxId
        self.addresses = addresses
        self.loyaltyPoints = loyaltyPoints
        self.createdAt = createdAt
    }
}
