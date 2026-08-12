// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/FareQuote.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `FareQuoteResponse` (`GET /v1/customer/fare-quote`) and
//   `EtaResponse` (`GET /v1/customer/orders/{id}/eta`). Both are
//   non-binding estimates — the server always recomputes the authoritative
//   fare at order-creation time, and the ETA has no real traffic data.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public struct FareQuote: Codable, Sendable, Equatable {
    public let fareAmountMinor: Int64
    public let currency: String
    public let distanceKm: Double

    public init(fareAmountMinor: Int64, currency: String, distanceKm: Double) {
        self.fareAmountMinor = fareAmountMinor
        self.currency = currency
        self.distanceKm = distanceKm
    }

    public var money: Money { Money(amountMinor: fareAmountMinor, currency: currency) }
}

public struct EtaInfo: Codable, Sendable, Equatable {
    public let etaMinutes: Double
    public let distanceKm: Double

    public init(etaMinutes: Double, distanceKm: Double) {
        self.etaMinutes = etaMinutes
        self.distanceKm = distanceKm
    }
}
