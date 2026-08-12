// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/Location.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `qervon_domain::Location` and the `AddressDto` shape returned by
//   the API gateway (`backend/crates/api-contracts/src/lib.rs`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public struct GeoLocation: Codable, Sendable, Equatable {
    public let latitude: Double
    public let longitude: Double

    public init(latitude: Double, longitude: Double) {
        self.latitude = latitude
        self.longitude = longitude
    }
}

public struct Address: Codable, Sendable, Equatable {
    public let latitude: Double
    public let longitude: Double
    public let label: String?

    public init(latitude: Double, longitude: Double, label: String?) {
        self.latitude = latitude
        self.longitude = longitude
        self.label = label
    }
}
