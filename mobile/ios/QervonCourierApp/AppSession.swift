// =============================================================================
// File:           mobile/ios/QervonCourierApp/AppSession.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Composition root: owns the Keychain token store, the HTTP client, and
//   the typed API surface, and tracks the coarse session state the root
//   view switches on (logged out / biometric-locked / active).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import QervonCore
import QervonNetworking
import QervonSecurity

public enum SessionState: Equatable {
    case launching
    case loggedOut
    /// Valid tokens exist, but the user enabled biometric unlock and has not
    /// yet passed the local Face ID / Touch ID gate this launch.
    case locked
    case active
}

@MainActor
public final class AppSession: ObservableObject {
    public let tokenStore: KeychainTokenStore
    public let httpClient: HTTPClient
    public let api: QervonAPI
    public let biometricGate = BiometricGate()

    @Published public private(set) var state: SessionState = .launching
    @Published public private(set) var claims: AccessTokenClaims?

    public init() {
        let store = KeychainTokenStore()
        tokenStore = store
        httpClient = HTTPClient(baseURL: APIEnvironment.baseURL, tokenStore: store)
        api = QervonAPI(client: httpClient)
        bootstrap()
    }

    private func bootstrap() {
        guard let claims = tokenStore.currentClaims(), !claims.isExpired else {
            tokenStore.clear()
            state = .loggedOut
            return
        }
        self.claims = claims
        state = AppPreferences.shared.isBiometricUnlockEnabled ? .locked : .active
    }

    public func unlockWithBiometrics() async -> Bool {
        let success = await biometricGate.unlock(
            reason: "Qervon Kurye oturumunuzu açmak için kimliğinizi doğrulayın"
        )
        if success {
            state = .active
        }
        return success
    }

    public func continueWithoutBiometrics() {
        // Only reachable when the user explicitly declines/cancels the
        // biometric prompt but the stored session is still valid; falls
        // back to treating this launch as active without re-authenticating
        // against the backend, since the Keychain-backed tokens are still
        // the source of truth for API calls either way.
        state = .active
    }

    public func completeLogin(tokens: AuthTokens) throws {
        try tokenStore.save(tokens: tokens)
        claims = try AccessTokenClaims.decode(fromAccessToken: tokens.accessToken)
        state = .active
    }

    public func logout() async {
        if let tokens = tokenStore.currentTokens() {
            try? await api.logout(refreshToken: tokens.refreshToken)
        }
        tokenStore.clear()
        claims = nil
        state = .loggedOut
    }
}
