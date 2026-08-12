// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/DispatchFeature/DispatchHomeView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import QervonCore
import QervonNetworking
import QervonDesignSystem
import OrdersFeature

public struct DispatchHomeView: View {
    @StateObject private var viewModel: DispatchViewModel
    @State private var showingActiveOrder = false
    private let api: QervonAPI

    public init(api: QervonAPI) {
        self.api = api
        _viewModel = StateObject(wrappedValue: DispatchViewModel(api: api))
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.lg) {
                header
                gpsStatusCard

                if let activeOrder = viewModel.activeOrder {
                    ActiveJobCard(order: activeOrder) {
                        showingActiveOrder = true
                    }
                } else if let offer = viewModel.pendingOffer {
                    JobOfferCard(
                        offer: offer,
                        secondsRemaining: viewModel.secondsRemainingOnOffer,
                        isResponding: viewModel.isRespondingToOffer,
                        onAccept: { Task { await viewModel.acceptOffer() } },
                        onReject: { Task { await viewModel.rejectOffer() } }
                    )
                } else if viewModel.isOnline {
                    QervonCard {
                        VStack(spacing: QervonSpacing.sm) {
                            ProgressView().tint(QervonColor.accent)
                            Text("Yeni iş bekleniyor…")
                                .font(.system(size: 13))
                                .foregroundColor(QervonColor.textSecondary)
                        }
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, QervonSpacing.sm)
                    }
                    .padding(.horizontal, QervonSpacing.lg)
                } else {
                    QervonCard {
                        Text("Çevrimiçi olduğunuzda size en yakın iş burada görünecek.")
                            .font(.system(size: 13))
                            .foregroundColor(QervonColor.textSecondary)
                            .frame(maxWidth: .infinity)
                    }
                    .padding(.horizontal, QervonSpacing.lg)
                }

                if let errorMessage = viewModel.errorMessage {
                    Text(errorMessage)
                        .font(.system(size: 13))
                        .foregroundColor(QervonColor.danger)
                        .padding(.horizontal, QervonSpacing.lg)
                }

                onlineToggleButton
            }
            .padding(.vertical, QervonSpacing.lg)
        }
        .qervonScreenBackground()
        .onAppear { viewModel.onAppear() }
        .onDisappear { viewModel.onDisappear() }
        .sheet(isPresented: $showingActiveOrder, onDismiss: {
            Task { await viewModel.refreshAll() }
        }) {
            if let order = viewModel.activeOrder {
                NavigationStack {
                    ActiveOrderDetailView(order: order, api: api) {
                        viewModel.clearActiveOrder()
                        showingActiveOrder = false
                    }
                }
            }
        }
    }

    private var header: some View {
        HStack {
            VStack(alignment: .leading, spacing: 2) {
                Text("QERVON KURYE")
                    .font(.system(size: 16, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
                Text(viewModel.isOnline ? "Çevrimiçi" : "Çevrimdışı")
                    .font(.system(size: 12, weight: .semibold))
                    .foregroundColor(viewModel.isOnline ? QervonColor.success : QervonColor.textSecondary)
            }
            Spacer()
            Circle()
                .fill(viewModel.isOnline ? QervonColor.success : QervonColor.danger)
                .frame(width: 12, height: 12)
        }
        .padding(.horizontal, QervonSpacing.lg)
    }

    private var gpsStatusCard: some View {
        QervonCard {
            HStack {
                Text(viewModel.locationBroadcaster.statusText)
                    .font(.system(size: 12, weight: .semibold, design: .monospaced))
                    .foregroundColor(QervonColor.success)
                Spacer()
            }
        }
        .padding(.horizontal, QervonSpacing.lg)
    }

    private var onlineToggleButton: some View {
        Button(viewModel.isOnline ? "Çevrimdışı Ol" : "Çevrimiçi Ol") {
            Task { await viewModel.toggleOnline() }
        }
        .buttonStyle(QervonButtonStyle(kind: viewModel.isOnline ? .destructive : .primary))
        .disabled(viewModel.isTogglingOnline || viewModel.activeOrder != nil)
        .padding(.horizontal, QervonSpacing.lg)
    }
}
