// =============================================================================
// File:           mobile/ios/QervonCustomerApp/QervonCustomerApp.swift
// Project:        Qervon Logistics Operating System (LOS)
// Author:         USDTG GROUP TECHNOLOGY LLC / Irfan Gedik
// Description:    SwiftUI Application Entry Point for iOS Customer App
// =============================================================================

import SwiftUI

@main
struct QervonCustomerApp: App {
    var body: some Scene {
        WindowGroup {
            CustomerView()
        }
    }
}
