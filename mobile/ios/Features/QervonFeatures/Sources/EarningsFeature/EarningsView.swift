// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/EarningsFeature/EarningsView.swift
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

public struct EarningsView: View {
    @StateObject private var viewModel: EarningsViewModel

    public init(api: QervonAPI) {
        _viewModel = StateObject(wrappedValue: EarningsViewModel(api: api))
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.lg) {
                Text("Kazançlar")
                    .font(.system(size: 18, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
                    .padding(.top, QervonSpacing.lg)

                if let wallet = viewModel.wallet {
                    balanceCard(wallet: wallet)
                    periodRow
                    statsCard
                    transactionsCard(wallet: wallet)
                } else if viewModel.isLoading {
                    ProgressView().tint(QervonColor.accent).padding(.top, QervonSpacing.xl)
                } else if let errorMessage = viewModel.errorMessage {
                    Text(errorMessage)
                        .font(.system(size: 13))
                        .foregroundColor(QervonColor.danger)
                }
            }
            .padding(.horizontal, QervonSpacing.lg)
            .padding(.bottom, QervonSpacing.xl)
        }
        .qervonScreenBackground()
        .refreshable { await viewModel.load() }
        .task { await viewModel.load() }
    }

    private func balanceCard(wallet: CourierWallet) -> some View {
        QervonCard(accentBorder: QervonColor.success) {
            VStack(spacing: QervonSpacing.xs) {
                Text("BAKİYE")
                    .font(.system(size: 11, weight: .bold))
                    .foregroundColor(QervonColor.textSecondary)
                Text(wallet.balance.formatted)
                    .font(.system(size: 32, weight: .bold))
                    .foregroundColor(QervonColor.success)
            }
            .frame(maxWidth: .infinity)
        }
    }

    private var periodRow: some View {
        HStack(spacing: QervonSpacing.sm) {
            periodCard(title: "Bugün", money: viewModel.todayEarnings)
            periodCard(title: "Bu Hafta", money: viewModel.weekEarnings)
            periodCard(title: "Bu Ay", money: viewModel.monthEarnings)
        }
    }

    private func periodCard(title: String, money: Money?) -> some View {
        QervonCard {
            VStack(spacing: QervonSpacing.xs) {
                Text(title)
                    .font(.system(size: 11, weight: .semibold))
                    .foregroundColor(QervonColor.textSecondary)
                Text(money?.formatted ?? "—")
                    .font(.system(size: 14, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
            }
            .frame(maxWidth: .infinity)
        }
    }

    private var statsCard: some View {
        QervonCard {
            HStack {
                statColumn(title: "Teslimat", value: "\(viewModel.wallet?.deliveryCount ?? 0)")
                Divider().background(QervonColor.border)
                statColumn(
                    title: "Puan",
                    value: viewModel.averageRating.map { String(format: "%.1f ★", $0) } ?? "Henüz yok"
                )
            }
        }
    }

    private func statColumn(title: String, value: String) -> some View {
        VStack(spacing: QervonSpacing.xs) {
            Text(value)
                .font(.system(size: 16, weight: .bold))
                .foregroundColor(QervonColor.textPrimary)
            Text(title)
                .font(.system(size: 11))
                .foregroundColor(QervonColor.textSecondary)
        }
        .frame(maxWidth: .infinity)
    }

    private func transactionsCard(wallet: CourierWallet) -> some View {
        VStack(alignment: .leading, spacing: QervonSpacing.sm) {
            Text("Son İşlemler")
                .font(.system(size: 13, weight: .bold))
                .foregroundColor(QervonColor.textSecondary)

            if wallet.transactions.isEmpty {
                QervonCard {
                    Text("Henüz işlem yok.")
                        .font(.system(size: 13))
                        .foregroundColor(QervonColor.textSecondary)
                }
            } else {
                ForEach(wallet.transactions.sorted(by: { $0.createdAt > $1.createdAt })) { transaction in
                    QervonCard {
                        HStack {
                            VStack(alignment: .leading, spacing: 2) {
                                Text(transaction.transactionType.displayName)
                                    .font(.system(size: 13, weight: .semibold))
                                    .foregroundColor(QervonColor.textPrimary)
                                Text(QervonFormat.dayAndTime(transaction.createdAt))
                                    .font(.system(size: 11))
                                    .foregroundColor(QervonColor.textSecondary)
                            }
                            Spacer()
                            Text((transaction.transactionType.isCredit ? "+" : "-") + transaction.money.formatted)
                                .font(.system(size: 14, weight: .bold))
                                .foregroundColor(transaction.transactionType.isCredit ? QervonColor.success : QervonColor.danger)
                        }
                    }
                }
            }
        }
    }
}
