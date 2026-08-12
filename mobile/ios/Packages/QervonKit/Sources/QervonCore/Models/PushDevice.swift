// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/PushDevice.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `DevicePushTokenResponse` (`POST/GET /v1/push/devices`). No
//   APNs sending is wired server-side yet (see BACKEND_BACKLOG.md) — this
//   only records where a push notification could be delivered.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public enum PushPlatform: String, Codable, Sendable, Equatable {
    case ios
    case android
}

public struct DevicePushToken: Codable, Sendable, Equatable, Identifiable {
    public let id: UUID
    public let platform: PushPlatform
    public let deviceToken: String
    public let createdAt: Date

    public init(id: UUID, platform: PushPlatform, deviceToken: String, createdAt: Date) {
        self.id = id
        self.platform = platform
        self.deviceToken = deviceToken
        self.createdAt = createdAt
    }
}
