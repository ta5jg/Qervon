// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/AuthFeature/BiometricLockView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Shown at launch when the courier previously enabled biometric unlock
//   and the stored session tokens are still valid. Purely a local gate —
//   see QervonSecurity/BiometricGate.swift for the honesty note about there
//   being no backend biometric API.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import QervonSecurity
import QervonDesignSystem

public struct BiometricLockView: View {
    private let gate = BiometricGate()
    private let onUnlock: () async -> Bool
    private let onContinueWithoutBiometrics: () -> Void
    @State private var isAttempting = false
    @State private var errorMessage: String?

    public init(onUnlock: @escaping () async -> Bool, onContinueWithoutBiometrics: @escaping () -> Void) {
        self.onUnlock = onUnlock
        self.onContinueWithoutBiometrics = onContinueWithoutBiometrics
    }

    public var body: some View {
        VStack(spacing: QervonSpacing.lg) {
            Spacer()
            Image(systemName: gate.availableKind() == .faceID ? "faceid" : "touchid")
                .font(.system(size: 56))
                .foregroundColor(QervonColor.accent)
            Text("Qervon Kurye Kilitli")
                .font(.system(size: 20, weight: .bold))
                .foregroundColor(QervonColor.textPrimary)
            Text("Oturumunuzu açmak için kimliğinizi doğrulayın.")
                .font(.system(size: 14))
                .foregroundColor(QervonColor.textSecondary)
            if let errorMessage {
                Text(errorMessage)
                    .font(.system(size: 13))
                    .foregroundColor(QervonColor.danger)
            }
            Button("Kimliği Doğrula") {
                Task { await attempt() }
            }
            .buttonStyle(QervonButtonStyle(isEnabled: !isAttempting))
            .disabled(isAttempting)
            .padding(.horizontal, QervonSpacing.xl)
            Spacer()
        }
        .qervonScreenBackground()
        .task { await attempt() }
    }

    private func attempt() async {
        isAttempting = true
        let success = await onUnlock()
        isAttempting = false
        if !success {
            errorMessage = "Doğrulama başarısız oldu, tekrar deneyin."
        }
    }
}
