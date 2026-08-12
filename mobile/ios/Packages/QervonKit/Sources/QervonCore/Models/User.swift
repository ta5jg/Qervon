// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/User.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `UserResponse` (returned by `POST /v1/auth/phone`, among others).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public struct QervonUser: Codable, Sendable, Equatable, Identifiable {
    public let id: UUID
    public let email: String
    public let displayName: String
    public let role: String
    public let status: String
    public let createdAt: Date

    public init(id: UUID, email: String, displayName: String, role: String, status: String, createdAt: Date) {
        self.id = id
        self.email = email
        self.displayName = displayName
        self.role = role
        self.status = status
        self.createdAt = createdAt
    }
}
