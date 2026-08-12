// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/AppNotification.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `qervon_domain::Notification` (`GET /v1/customer/notifications`).
//   Named `AppNotification` to avoid clashing with Foundation's
//   `Notification`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public enum NotificationChannel: String, Codable, Sendable, Equatable {
    case push
    case sms
    case email
    case whatsapp

    public var displayName: String {
        switch self {
        case .push: return "Push"
        case .sms: return "SMS"
        case .email: return "E-posta"
        case .whatsapp: return "WhatsApp"
        }
    }
}

public enum NotificationDeliveryStatus: String, Codable, Sendable, Equatable {
    case queued
    case sent
    case failed
    case read
}

public struct AppNotification: Codable, Sendable, Equatable, Identifiable {
    public let id: UUID
    public let recipientId: UUID
    public let channel: NotificationChannel
    public let title: String
    public let body: String
    public let status: NotificationDeliveryStatus
    public let createdAt: Date
    public let sentAt: Date?

    public init(
        id: UUID,
        recipientId: UUID,
        channel: NotificationChannel,
        title: String,
        body: String,
        status: NotificationDeliveryStatus,
        createdAt: Date,
        sentAt: Date?
    ) {
        self.id = id
        self.recipientId = recipientId
        self.channel = channel
        self.title = title
        self.body = body
        self.status = status
        self.createdAt = createdAt
        self.sentAt = sentAt
    }
}
