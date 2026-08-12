// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/Courier.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `CourierResponse` (`backend/crates/api-contracts/src/lib.rs`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public enum VehicleType: String, Codable, Sendable, Equatable, CaseIterable {
    case bicycle
    case motorcycle
    case car

    public var displayName: String {
        switch self {
        case .bicycle: return "Bisiklet"
        case .motorcycle: return "Motosiklet"
        case .car: return "Otomobil"
        }
    }
}

public enum CourierStatus: String, Codable, Sendable, Equatable {
    case available
    case busy
    case offline
}

public struct Courier: Codable, Sendable, Equatable, Identifiable {
    public let id: UUID
    public let name: String
    public let vehicle: VehicleType
    public let status: CourierStatus
    public let currentLocation: GeoLocation?
    public let registeredAt: Date

    public init(
        id: UUID,
        name: String,
        vehicle: VehicleType,
        status: CourierStatus,
        currentLocation: GeoLocation?,
        registeredAt: Date
    ) {
        self.id = id
        self.name = name
        self.vehicle = vehicle
        self.status = status
        self.currentLocation = currentLocation
        self.registeredAt = registeredAt
    }
}
