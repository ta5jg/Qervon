// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonSecurity/BiometricGate.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Local biometric (Face ID / Touch ID) gate for unlocking an already
//   stored session. There is no backend biometric API — this only guards
//   local access to the tokens already sitting in the Keychain; it does
//   not authenticate with the server in any way.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import LocalAuthentication

public enum BiometricKind: Sendable {
    case none
    case touchID
    case faceID
}

public struct BiometricGate: Sendable {
    public init() {}

    public func availableKind() -> BiometricKind {
        let context = LAContext()
        var error: NSError?
        guard context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &error) else {
            return .none
        }
        switch context.biometryType {
        case .faceID: return .faceID
        case .touchID: return .touchID
        default: return .none
        }
    }

    /// Prompts the user to unlock with biometrics. Returns `true` on
    /// success, `false` on cancellation/failure (never throws for a normal
    /// user cancel — callers should treat `false` as "stay on the lock
    /// screen", not as an error to surface).
    public func unlock(reason: String) async -> Bool {
        let context = LAContext()
        var policyError: NSError?
        guard context.canEvaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, error: &policyError) else {
            return false
        }
        return await withCheckedContinuation { continuation in
            context.evaluatePolicy(.deviceOwnerAuthenticationWithBiometrics, localizedReason: reason) { success, _ in
                continuation.resume(returning: success)
            }
        }
    }
}
