// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/CustomerOrderFeature/OrderHistoryView.swift
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

public struct OrderHistoryView: View {
    @StateObject private var viewModel: OrderHistoryViewModel
    private let api: QervonAPI

    public init(api: QervonAPI) {
        self.api = api
        _viewModel = StateObject(wrappedValue: OrderHistoryViewModel(api: api))
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.lg) {
                Text("Siparişlerim")
                    .font(.system(size: 20, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
                    .padding(.top, QervonSpacing.lg)

                if !viewModel.activeOrders.isEmpty {
                    section(title: "Aktif", orders: viewModel.activeOrders)
                }
                if !viewModel.pastOrders.isEmpty {
                    section(title: "Geçmiş", orders: viewModel.pastOrders)
                }
                if viewModel.orders.isEmpty && !viewModel.isLoading {
                    QervonCard {
                        Text("Henüz siparişiniz yok.")
                            .font(.system(size: 13))
                            .foregroundColor(QervonColor.textSecondary)
                    }
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

    private func section(title: String, orders: [Order]) -> some View {
        VStack(alignment: .leading, spacing: QervonSpacing.sm) {
            Text(title.uppercased())
                .font(.system(size: 12, weight: .bold))
                .foregroundColor(QervonColor.textSecondary)
                .padding(.horizontal, QervonSpacing.lg)

            ForEach(orders) { order in
                NavigationLink {
                    OrderDetailView(order: order, api: api)
                } label: {
                    orderRow(order)
                }
                .padding(.horizontal, QervonSpacing.lg)
            }
        }
    }

    private func orderRow(_ order: Order) -> some View {
        QervonCard {
            HStack {
                VStack(alignment: .leading, spacing: 2) {
                    Text(order.dropoff.label ?? "Teslim noktası")
                        .font(.system(size: 14, weight: .bold))
                        .foregroundColor(QervonColor.textPrimary)
                    Text(QervonFormat.dayAndTime(order.createdAt))
                        .font(.system(size: 11))
                        .foregroundColor(QervonColor.textSecondary)
                }
                Spacer()
                VStack(alignment: .trailing, spacing: 2) {
                    Text(order.fare.formatted)
                        .font(.system(size: 13, weight: .bold))
                        .foregroundColor(QervonColor.textPrimary)
                    Text(order.status.displayName)
                        .font(.system(size: 11, weight: .semibold))
                        .foregroundColor(statusColor(order.status))
                }
            }
        }
    }

    private func statusColor(_ status: OrderStatus) -> Color {
        switch status {
        case .delivered: return QervonColor.success
        case .cancelled, .returned: return QervonColor.danger
        case .pending: return QervonColor.warning
        case .courierAssigned, .inTransit: return QervonColor.accent
        }
    }
}
