// =============================================================================
// File:           mobile/ios/QervonCourierApp/QervonCourierApp.swift
// Project:        Qervon Logistics Operating System (LOS)
// Author:         USDTG GROUP TECHNOLOGY LLC / Irfan Gedik
// Description:    SwiftUI Application Entry Point for iOS Courier Terminal
// =============================================================================

import SwiftUI

@main
struct QervonCourierApp: App {
    var body: some Scene {
        WindowGroup {
            CourierView()
        }
    }
}
