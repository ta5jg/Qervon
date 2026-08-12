// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/Money.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors the `MoneyDto` shape (`{amount_minor, currency}`). Amounts are
//   always minor units (kuruş), matching the backend's `Money` value object.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public struct Money: Codable, Sendable, Equatable {
    public let amountMinor: Int64
    public let currency: String

    public init(amountMinor: Int64, currency: String) {
        self.amountMinor = amountMinor
        self.currency = currency
    }

    /// Formats as a localized currency string, e.g. "₺45,00".
    public var formatted: String {
        let value = Double(amountMinor) / 100.0
        let formatter = NumberFormatter()
        formatter.numberStyle = .currency
        formatter.currencyCode = currency
        formatter.locale = Locale(identifier: "tr_TR")
        return formatter.string(from: NSNumber(value: value)) ?? "\(value) \(currency)"
    }
}
