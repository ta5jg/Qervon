// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/LocationSnapshot.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors the API gateway's `LocationUpdateEvent`
//   (`GET /v1/orders/{id}/tracking`) — a single courier location sample.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public struct LocationSnapshot: Codable, Sendable, Equatable {
    public let courierId: UUID
    public let tenantId: UUID
    public let latitude: Double
    public let longitude: Double
    public let timestamp: Date
    public let fraudFlagged: Bool
    public let fraudRiskScore: Double

    public init(
        courierId: UUID,
        tenantId: UUID,
        latitude: Double,
        longitude: Double,
        timestamp: Date,
        fraudFlagged: Bool,
        fraudRiskScore: Double
    ) {
        self.courierId = courierId
        self.tenantId = tenantId
        self.latitude = latitude
        self.longitude = longitude
        self.timestamp = timestamp
        self.fraudFlagged = fraudFlagged
        self.fraudRiskScore = fraudRiskScore
    }

    public var coordinate: GeoLocation { GeoLocation(latitude: latitude, longitude: longitude) }
}
