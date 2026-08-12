// =============================================================================
// File:           mobile/ios/QervonCustomerApp/QervonCustomerApp.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   SwiftUI application entry point for the native Qervon Customer app.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI

@main
struct QervonCustomerApp: App {
    @StateObject private var session = AppSession()
    @UIApplicationDelegateAdaptor(AppDelegate.self) private var appDelegate

    var body: some Scene {
        WindowGroup {
            RootView()
                .environmentObject(session)
                .preferredColorScheme(.dark)
                .task {
                    appDelegate.api = session.api
                }
        }
    }
}
