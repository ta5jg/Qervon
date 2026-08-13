// =============================================================================
// File:           mobile/ios/QervonCourierApp/MainTabView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   The signed-in courier's tab bar: Home (dispatch/online), Kazanç
//   (earnings/stats), Profil (account/settings/logout).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import QervonDesignSystem
import DispatchFeature
import OrdersFeature
import EarningsFeature
import ProfileFeature

struct MainTabView: View {
    @EnvironmentObject private var session: AppSession

    var body: some View {
        TabView {
            NavigationStack {
                DispatchHomeView(api: session.api)
            }
            .tabItem { Label("Panel", systemImage: "dot.radiowaves.left.and.right") }

            NavigationStack {
                CourierOrdersView(api: session.api)
            }
            .tabItem { Label("İşlerim", systemImage: "list.bullet.rectangle.fill") }

            NavigationStack {
                EarningsView(api: session.api)
            }
            .tabItem { Label("Kazanç", systemImage: "wallet.pass.fill") }

            NavigationStack {
                ProfileView(api: session.api) {
                    Task { await session.logout() }
                }
            }
            .tabItem { Label("Hesap", systemImage: "person.fill") }
        }
        .tint(QervonColor.accent)
    }
}
