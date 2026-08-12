// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonSecurity/TokenStore.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Keychain-backed storage for the access/refresh token pair. Conforms to
//   `AuthTokenStoring` (declared in QervonCore) so `QervonNetworking` can
//   consume it without a package dependency cycle.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import Security
import QervonCore

public enum TokenStoreError: Error {
    case keychainWrite(OSStatus)
}

/// Thread-safe, Keychain-backed session token storage. A single instance
/// should be shared across the app (injected into `HTTPClient` and read by
/// feature view models to check session state at launch).
public final class KeychainTokenStore: AuthTokenStoring, @unchecked Sendable {
    private let service: String
    private let account = "qervon.session"
    private let lock = NSLock()
    private var cachedTokens: AuthTokens?

    public init(service: String = "com.qervon.ios.courier") {
        self.service = service
        self.cachedTokens = Self.readFromKeychain(service: service, account: account)
    }

    public func currentTokens() -> AuthTokens? {
        lock.lock()
        defer { lock.unlock() }
        return cachedTokens
    }

    public func currentClaims() -> AccessTokenClaims? {
        guard let tokens = currentTokens() else { return nil }
        return try? AccessTokenClaims.decode(fromAccessToken: tokens.accessToken)
    }

    public func save(tokens: AuthTokens) throws {
        let data = try JSONEncoder().encode(tokens)
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: service,
            kSecAttrAccount: account,
        ]
        SecItemDelete(query as CFDictionary)
        var attributes = query
        attributes[kSecValueData] = data
        attributes[kSecAttrAccessible] = kSecAttrAccessibleAfterFirstUnlock
        let status = SecItemAdd(attributes as CFDictionary, nil)
        guard status == errSecSuccess else {
            throw TokenStoreError.keychainWrite(status)
        }
        lock.lock()
        cachedTokens = tokens
        lock.unlock()
    }

    public func clear() {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: service,
            kSecAttrAccount: account,
        ]
        SecItemDelete(query as CFDictionary)
        lock.lock()
        cachedTokens = nil
        lock.unlock()
    }

    private static func readFromKeychain(service: String, account: String) -> AuthTokens? {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrService: service,
            kSecAttrAccount: account,
            kSecReturnData: true,
            kSecMatchLimit: kSecMatchLimitOne,
        ]
        var result: AnyObject?
        guard SecItemCopyMatching(query as CFDictionary, &result) == errSecSuccess,
              let data = result as? Data
        else {
            return nil
        }
        return try? JSONDecoder().decode(AuthTokens.self, from: data)
    }
}
