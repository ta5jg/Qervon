// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/OrdersFeature/CourierOrdersView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-13
// Version:        0.1.0
//
// Description:
//   Courier "İşlerim" tab with live active-order list and detail navigation.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import QervonCore
import QervonNetworking
import QervonDesignSystem

public struct CourierOrdersView: View {
    @StateObject private var viewModel: CourierOrdersViewModel
    private let api: QervonAPI

    public init(api: QervonAPI) {
        self.api = api
        _viewModel = StateObject(wrappedValue: CourierOrdersViewModel(api: api))
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.lg) {
                Text("Aktif İşlerim")
                    .font(.system(size: 20, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
                    .padding(.top, QervonSpacing.lg)

                if viewModel.orders.isEmpty && !viewModel.isLoading {
                    QervonCard {
                        Text("Şu anda atanmış aktif iş yok.")
                            .font(.system(size: 13))
                            .foregroundColor(QervonColor.textSecondary)
                    }
                    .padding(.horizontal, QervonSpacing.lg)
                } else {
                    ForEach(viewModel.orders) { order in
                        NavigationLink {
                            ActiveOrderDetailView(order: order, api: api) {
                                Task { await viewModel.load() }
                            }
                        } label: {
                            orderRow(order)
                        }
                        .padding(.horizontal, QervonSpacing.lg)
                    }
                }

                if let errorMessage = viewModel.errorMessage {
                    Text(errorMessage)
                        .font(.system(size: 12))
                        .foregroundColor(QervonColor.danger)
                        .padding(.horizontal, QervonSpacing.lg)
                }
            }
            .padding(.bottom, QervonSpacing.xl)
        }
        .qervonScreenBackground()
        .refreshable { await viewModel.load() }
        .task { await viewModel.load() }
        .onAppear { viewModel.startLiveUpdates() }
        .onDisappear { viewModel.stopLiveUpdates() }
    }

    private func orderRow(_ order: Order) -> some View {
        QervonCard {
            VStack(alignment: .leading, spacing: QervonSpacing.xs) {
                HStack {
                    Text(order.status.displayName.uppercased())
                        .font(.system(size: 11, weight: .bold))
                        .foregroundColor(order.status == .inTransit ? QervonColor.accent : QervonColor.warning)
                    Spacer()
                    Text(order.fare.formatted)
                        .font(.system(size: 13, weight: .bold))
                        .foregroundColor(QervonColor.success)
                }
                Text("Alım: \(order.pickup.label ?? "Konum")")
                    .font(.system(size: 12))
                    .foregroundColor(QervonColor.textSecondary)
                Text("Teslim: \(order.dropoff.label ?? "Konum")")
                    .font(.system(size: 12))
                    .foregroundColor(QervonColor.textSecondary)
            }
        }
    }
}
