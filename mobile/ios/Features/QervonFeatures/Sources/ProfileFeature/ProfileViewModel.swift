// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/ProfileFeature/ProfileViewModel.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import UIKit
import UserNotifications
import QervonCore
import QervonNetworking
import QervonSecurity

@MainActor
public final class ProfileViewModel: ObservableObject {
    @Published public private(set) var courier: Courier?
    @Published public var phoneInput = ""
    @Published public private(set) var isLinkingPhone = false
    @Published public private(set) var phoneLinked = false
    @Published public var errorMessage: String?
    @Published public var isBiometricEnabled: Bool {
        didSet { AppPreferences.shared.isBiometricUnlockEnabled = isBiometricEnabled }
    }

    public let biometricGate = BiometricGate()
    private let api: QervonAPI

    public init(api: QervonAPI) {
        self.api = api
        self.isBiometricEnabled = AppPreferences.shared.isBiometricUnlockEnabled
    }

    public func load() async {
        courier = try? await api.getOwnCourier()
    }

    public func linkPhone() async {
        guard !phoneInput.trimmingCharacters(in: .whitespaces).isEmpty else { return }
        isLinkingPhone = true
        errorMessage = nil
        defer { isLinkingPhone = false }
        do {
            _ = try await api.linkPhone(phoneInput)
            phoneLinked = true
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    /// Requests local notification permission and, if granted, asks iOS for
    /// a remote-notification (APNs) device token. Whether that token is
    /// ever actually usable depends on entitlements/provisioning outside
    /// this app's control — the App target only calls
    /// `registerPushDevice` once (and if) a real token comes back.
    public func requestPushPermission() async -> Bool {
        do {
            let granted = try await UNUserNotificationCenter.current()
                .requestAuthorization(options: [.alert, .sound, .badge])
            if granted {
                UIApplication.shared.registerForRemoteNotifications()
            }
            return granted
        } catch {
            errorMessage = error.localizedDescription
            return false
        }
    }
}
