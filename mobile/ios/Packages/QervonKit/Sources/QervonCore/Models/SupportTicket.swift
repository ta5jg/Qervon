// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/SupportTicket.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `SupportTicketResponse` (`POST/GET /v1/customer/support-tickets`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public enum TicketStatus: String, Codable, Sendable, Equatable {
    case open
    case inProgress = "in_progress"
    case resolved
    case closed

    public var displayName: String {
        switch self {
        case .open: return "Açık"
        case .inProgress: return "İşlemde"
        case .resolved: return "Çözüldü"
        case .closed: return "Kapatıldı"
        }
    }
}

public struct SupportTicket: Codable, Sendable, Equatable, Identifiable {
    public let id: UUID
    public let customerId: UUID
    public let orderId: UUID?
    public let subject: String
    public let message: String
    public let status: TicketStatus
    public let createdAt: Date

    public init(
        id: UUID,
        customerId: UUID,
        orderId: UUID?,
        subject: String,
        message: String,
        status: TicketStatus,
        createdAt: Date
    ) {
        self.id = id
        self.customerId = customerId
        self.orderId = orderId
        self.subject = subject
        self.message = message
        self.status = status
        self.createdAt = createdAt
    }
}
