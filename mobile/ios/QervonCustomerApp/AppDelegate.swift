// =============================================================================
// File:           mobile/ios/QervonCustomerApp/AppDelegate.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Captures a real APNs device token (when the OS actually grants one —
//   it never will on the Simulator or without a signed build that has the
//   Push Notifications capability/entitlement and a paid Apple Developer
//   account) and registers it with the backend. No fabricated token is
//   ever sent — see BACKEND_BACKLOG.md for the native-push scope boundary.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import UIKit
import QervonNetworking
import QervonCore

final class AppDelegate: NSObject, UIApplicationDelegate {
    var api: QervonAPI?

    func application(
        _ application: UIApplication,
        didFinishLaunchingWithOptions launchOptions: [UIApplication.LaunchOptionsKey: Any]? = nil
    ) -> Bool {
        true
    }

    func application(
        _ application: UIApplication,
        didRegisterForRemoteNotificationsWithDeviceToken deviceToken: Data
    ) {
        let token = deviceToken.map { String(format: "%02.2hhx", $0) }.joined()
        guard let api else { return }
        Task {
            try? await api.registerPushDevice(platform: .ios, appVariant: .customer, deviceToken: token)
        }
    }

    func application(
        _ application: UIApplication,
        didFailToRegisterForRemoteNotificationsWithError error: Error
    ) {
        // Expected in the Simulator and on any build without the Push
        // Notifications entitlement — intentionally not surfaced to the user.
    }
}
