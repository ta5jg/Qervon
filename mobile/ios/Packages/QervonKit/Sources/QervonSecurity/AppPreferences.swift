// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonSecurity/AppPreferences.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Small, non-sensitive local preferences (UserDefaults-backed). The
//   biometric on/off toggle itself is not a secret; the tokens it gates
//   live in the Keychain (`KeychainTokenStore`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public final class AppPreferences: @unchecked Sendable {
    public static let shared = AppPreferences()

    private let defaults: UserDefaults
    private let biometricEnabledKey = "qervon.biometric_unlock_enabled"
    private let lastTenantSlugKey = "qervon.last_tenant_slug"

    public init(defaults: UserDefaults = .standard) {
        self.defaults = defaults
    }

    public var isBiometricUnlockEnabled: Bool {
        get { defaults.bool(forKey: biometricEnabledKey) }
        set { defaults.set(newValue, forKey: biometricEnabledKey) }
    }

    /// Remembered for convenience so the login screen can pre-fill it; a
    /// courier's tenant rarely changes, but the field always stays editable.
    public var lastTenantSlug: String? {
        get { defaults.string(forKey: lastTenantSlugKey) }
        set { defaults.set(newValue, forKey: lastTenantSlugKey) }
    }
}
