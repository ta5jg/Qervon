// =============================================================================
// File:           mobile/ios/QervonCustomerApp/MainTabView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   The signed-in customer's tab bar: Sipariş Ver (new order), Siparişlerim
//   (history/tracking), Profil (account/settings/logout).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import QervonDesignSystem
import CustomerOrderFeature
import CustomerProfileFeature

struct MainTabView: View {
    @EnvironmentObject private var session: AppSession
    @State private var selectedTab = Tab.newOrder
    @State private var historyRefreshToken = UUID()

    enum Tab {
        case newOrder
        case history
        case profile
    }

    var body: some View {
        TabView(selection: $selectedTab) {
            NavigationStack {
                NewOrderView(api: session.api) { _ in
                    // Jump to the history tab so the customer can see (and
                    // start tracking) the order they just created.
                    historyRefreshToken = UUID()
                    selectedTab = .history
                }
            }
            .tabItem { Label("Sipariş Ver", systemImage: "shippingbox.fill") }
            .tag(Tab.newOrder)

            NavigationStack {
                OrderHistoryView(api: session.api)
                    .id(historyRefreshToken)
            }
            .tabItem { Label("Siparişlerim", systemImage: "list.bullet.rectangle.fill") }
            .tag(Tab.history)

            NavigationStack {
                CustomerProfileView(api: session.api) {
                    Task { await session.logout() }
                }
            }
            .tabItem { Label("Profil", systemImage: "person.fill") }
            .tag(Tab.profile)
        }
        .tint(QervonColor.accent)
    }
}
