// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/PushDevice.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `DevicePushTokenResponse` (`POST/GET /v1/push/devices`). iOS/APNs
//   delivery is real server-side as of 2026-08-16 when `APNS_*` env vars are
//   configured (see backend `apns.rs`); Android/FCM is still unwired (see
//   BACKEND_BACKLOG.md). `appVariant` tells the backend which bundle id's
//   `apns-topic` this token belongs to — required because the Courier and
//   Customer apps have distinct bundle identifiers.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public enum PushPlatform: String, Codable, Sendable, Equatable {
    case ios
    case android
}

public enum AppVariant: String, Codable, Sendable, Equatable {
    case courier
    case customer
}

public struct DevicePushToken: Codable, Sendable, Equatable, Identifiable {
    public let id: UUID
    public let platform: PushPlatform
    public let appVariant: AppVariant
    public let deviceToken: String
    public let createdAt: Date

    public init(id: UUID, platform: PushPlatform, appVariant: AppVariant, deviceToken: String, createdAt: Date) {
        self.id = id
        self.platform = platform
        self.appVariant = appVariant
        self.deviceToken = deviceToken
        self.createdAt = createdAt
    }
}
