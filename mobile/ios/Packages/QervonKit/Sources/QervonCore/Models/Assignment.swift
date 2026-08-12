// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/Assignment.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `PendingOfferResponse` (`GET /v1/courier/me/offer`) — a job
//   offered to the signed-in courier that has not yet been accepted or
//   rejected.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public struct PendingOffer: Codable, Sendable, Equatable, Identifiable {
    public var id: UUID { assignmentId }
    public let assignmentId: UUID
    public let order: Order
    public let offeredAt: Date
    public let expiresAt: Date

    public init(assignmentId: UUID, order: Order, offeredAt: Date, expiresAt: Date) {
        self.assignmentId = assignmentId
        self.order = order
        self.offeredAt = offeredAt
        self.expiresAt = expiresAt
    }

    /// Seconds remaining before the backend will lazily expire this offer.
    /// Never negative — callers should treat <= 0 as "already gone".
    public func secondsRemaining(now: Date = Date()) -> TimeInterval {
        max(0, expiresAt.timeIntervalSince(now))
    }
}
