// =============================================================================
// File:           mobile/ios/QervonCourierApp/RootView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Switches between the auth flow, the biometric lock screen, and the main
//   tab experience based on `AppSession.state`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import QervonDesignSystem
import AuthFeature

struct RootView: View {
    @EnvironmentObject private var session: AppSession

    var body: some View {
        Group {
            switch session.state {
            case .launching:
                ProgressView()
                    .tint(QervonColor.accent)
                    .qervonScreenBackground()
            case .loggedOut:
                LoginView(api: session.api) { tokens in
                    try? session.completeLogin(tokens: tokens)
                }
            case .locked:
                BiometricLockView(
                    onUnlock: { await session.unlockWithBiometrics() },
                    onContinueWithoutBiometrics: { session.continueWithoutBiometrics() }
                )
            case .active:
                MainTabView()
            }
        }
    }
}
