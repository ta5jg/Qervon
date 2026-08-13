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
                courierShell {
                    DispatchHomeView(api: session.api)
                }
            }
            .tabItem { Label("Navigasyon", systemImage: "location.fill") }

            NavigationStack {
                courierShell {
                    CourierOrdersView(api: session.api)
                }
            }
            .tabItem { Label("POD / İmza", systemImage: "checkmark.seal.fill") }

            NavigationStack {
                courierShell {
                    EarningsView(api: session.api)
                }
            }
            .tabItem { Label("Kazançlar", systemImage: "wallet.pass.fill") }

            NavigationStack {
                courierShell {
                    ProfileView(api: session.api) {
                        Task { await session.logout() }
                    }
                }
            }
            .tabItem { Label("Profil", systemImage: "person.fill") }
        }
        .tint(QervonColor.accent)
    }

    @ViewBuilder
    private func courierShell<Content: View>(@ViewBuilder content: () -> Content) -> some View {
        VStack(spacing: 0) {
            QervonTerminalHeader(
                title: "KURYE TERMİNALİ",
                subtitle: "HARDWARE GPS BROADCASTER",
                badge: "OTURUM"
            )
            content()
        }
        .toolbar(.hidden, for: .navigationBar)
    }
}
