// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/Rating.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `CustomerRatingResponse` (`GET /v1/courier/me/ratings`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public struct CustomerRating: Codable, Sendable, Equatable, Identifiable {
    public let id: UUID
    public let orderId: UUID
    public let customerId: UUID
    public let courierId: UUID
    public let ratingStars: Int
    public let comment: String?
    public let photoUrl: String?
    public let createdAt: Date

    public init(
        id: UUID,
        orderId: UUID,
        customerId: UUID,
        courierId: UUID,
        ratingStars: Int,
        comment: String?,
        photoUrl: String?,
        createdAt: Date
    ) {
        self.id = id
        self.orderId = orderId
        self.customerId = customerId
        self.courierId = courierId
        self.ratingStars = ratingStars
        self.comment = comment
        self.photoUrl = photoUrl
        self.createdAt = createdAt
    }
}

extension [CustomerRating] {
    /// Average of all star ratings, or `nil` when there are none yet — the
    /// UI must show "no ratings yet" rather than a fabricated 0.0.
    public var averageStars: Double? {
        guard !isEmpty else { return nil }
        let total = reduce(0) { $0 + $1.ratingStars }
        return Double(total) / Double(count)
    }
}
