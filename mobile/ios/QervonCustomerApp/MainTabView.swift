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
    @State private var selectedTab = Tab.launchTab
    @State private var historyRefreshToken = UUID()

    enum Tab {
        case liveTracking
        case newOrder
        case history
        case profile
        case support

        static var launchTab: Tab {
            let args = ProcessInfo.processInfo.arguments
            guard let raw = args.first(where: { $0.hasPrefix("--qervon-customer-tab=") })?
                .split(separator: "=")
                .last
            else {
                return .liveTracking
            }
            switch String(raw) {
            case "order":
                return .newOrder
            case "history":
                return .history
            case "wallet":
                return .profile
            case "support":
                return .support
            default:
                return .liveTracking
            }
        }
    }

    var body: some View {
        TabView(selection: $selectedTab) {
            NavigationStack {
                customerShell {
                    CustomerLiveTrackingView(api: session.api)
                }
            }
            .tabItem { Label("Canlı Takip", systemImage: "location.fill") }
            .tag(Tab.liveTracking)

            NavigationStack {
                customerShell {
                    NewOrderView(api: session.api) { _ in
                        // Jump to the history tab so the customer can see (and
                        // start tracking) the order they just created.
                        historyRefreshToken = UUID()
                        selectedTab = .history
                    }
                }
            }
            .tabItem { Label("Sipariş Ver", systemImage: "shippingbox.fill") }
            .tag(Tab.newOrder)

            NavigationStack {
                customerShell {
                    OrderHistoryView(api: session.api)
                        .id(historyRefreshToken)
                }
            }
            .tabItem { Label("Geçmiş", systemImage: "clock.fill") }
            .tag(Tab.history)

            NavigationStack {
                customerShell {
                    CustomerProfileView(api: session.api) {
                        Task { await session.logout() }
                    }
                }
            }
            .tabItem { Label("Cüzdan", systemImage: "wallet.pass.fill") }
            .tag(Tab.profile)

            NavigationStack {
                customerShell {
                    CustomerSupportView(api: session.api)
                }
            }
            .tabItem { Label("Destek", systemImage: "message.fill") }
            .tag(Tab.support)
        }
        .tint(QervonColor.accent)
    }

    @ViewBuilder
    private func customerShell<Content: View>(@ViewBuilder content: () -> Content) -> some View {
        VStack(spacing: 0) {
            QervonTerminalHeader(
                title: "QERVON MÜŞTERİ",
                subtitle: "NATIVE iOS / ANDROID SIMULATOR",
                badge: "GPS LIVE"
            )
            content()
        }
        .toolbar(.hidden, for: .navigationBar)
    }
}
