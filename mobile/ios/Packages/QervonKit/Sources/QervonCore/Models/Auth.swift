// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/Auth.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `AuthResponse` and decodes the backend's custom
//   `qv1.<payload>.<signature>` access token locally (read-only: the app has
//   no signing secret and cannot verify the signature, it only reads the
//   claims it was already issued over HTTPS — see
//   `backend/apps/api-gateway/src/auth.rs`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public struct AuthTokens: Codable, Sendable, Equatable {
    public let accessToken: String
    public let refreshToken: String
    public let tokenType: String
    public let expiresInSeconds: Int

    public init(accessToken: String, refreshToken: String, tokenType: String, expiresInSeconds: Int) {
        self.accessToken = accessToken
        self.refreshToken = refreshToken
        self.tokenType = tokenType
        self.expiresInSeconds = expiresInSeconds
    }
}

public enum UserRole: String, Codable, Sendable, Equatable {
    case customer
    case company
    case courier
    case admin
    case superAdmin = "super_admin"
    case operator_ = "operator"
    case dispatcher
    case fleetManager = "fleet_manager"
    case support
}

/// Locally decoded claims from an access token, for display/expiry purposes.
public struct AccessTokenClaims: Sendable, Equatable {
    public let subject: UUID
    public let tenantId: UUID
    public let role: UserRole
    public let expiresAt: Date

    public var isExpired: Bool { expiresAt <= Date() }
}

private struct RawAccessClaims: Decodable {
    let subject: UUID
    let tenantId: UUID
    let role: UserRole
    let expiresAt: Int64

    enum CodingKeys: String, CodingKey {
        case subject
        case tenantId = "tenant_id"
        case role
        case expiresAt = "expires_at"
    }
}

public enum AccessTokenDecodingError: Error {
    case malformed
}

extension AccessTokenClaims {
    /// Decodes the `qv1.<payload>.<signature>` token format without
    /// verifying the signature (the client has no signing secret; it trusts
    /// the token because it just received it over HTTPS from the backend).
    public static func decode(fromAccessToken token: String) throws -> AccessTokenClaims {
        let parts = token.split(separator: ".", omittingEmptySubsequences: false)
        guard parts.count == 3, parts[0] == "qv1" else {
            throw AccessTokenDecodingError.malformed
        }
        guard let payload = base64URLDecode(String(parts[1])) else {
            throw AccessTokenDecodingError.malformed
        }
        let raw = try JSONDecoder().decode(RawAccessClaims.self, from: payload)
        return AccessTokenClaims(
            subject: raw.subject,
            tenantId: raw.tenantId,
            role: raw.role,
            expiresAt: Date(timeIntervalSince1970: TimeInterval(raw.expiresAt))
        )
    }
}

/// Implemented by `QervonSecurity`'s Keychain-backed store. Declared here
/// (dependency-free) so `QervonNetworking` can depend on the abstraction
/// without depending on `QervonSecurity`, avoiding a package cycle.
public protocol AuthTokenStoring: AnyObject, Sendable {
    func currentTokens() -> AuthTokens?
    func currentClaims() -> AccessTokenClaims?
    func save(tokens: AuthTokens) throws
    func clear()
}

private func base64URLDecode(_ value: String) -> Data? {
    var base64 = value.replacingOccurrences(of: "-", with: "+")
        .replacingOccurrences(of: "_", with: "/")
    let remainder = base64.count % 4
    if remainder > 0 {
        base64.append(String(repeating: "=", count: 4 - remainder))
    }
    return Data(base64Encoded: base64)
}
