// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/CustomerProfileFeature/CustomerSupportView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-13
// Version:        0.1.0
//
// Description:
//   Dedicated support center screen for customer app.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import QervonCore
import QervonNetworking
import QervonDesignSystem

public struct CustomerSupportView: View {
    @StateObject private var viewModel: CustomerSupportViewModel
    @State private var subject = "Mobil Destek Talebi"
    @State private var message = ""

    public init(api: QervonAPI) {
        _viewModel = StateObject(wrappedValue: CustomerSupportViewModel(api: api))
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.lg) {
                composerSection
                threadSection
            }
            .padding(.vertical, QervonSpacing.lg)
        }
        .qervonScreenBackground()
        .navigationTitle("Canlı Destek")
        .task { await viewModel.load() }
        .onAppear { viewModel.startLiveUpdates() }
        .onDisappear { viewModel.stopLiveUpdates() }
    }

    private var composerSection: some View {
        QervonCard {
            VStack(alignment: .leading, spacing: QervonSpacing.sm) {
                Text("Yeni Destek Talebi")
                    .font(.system(size: 15, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
                QervonTextField(title: "Konu", text: $subject)
                QervonTextField(title: "Mesaj", text: $message)
                Button("Talep Oluştur") {
                    Task {
                        await viewModel.submitTicket(subject: subject, message: message)
                        message = ""
                    }
                }
                .buttonStyle(
                    QervonButtonStyle(
                        isEnabled: !viewModel.isSubmitting &&
                            !subject.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty &&
                            !message.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                    )
                )
                .disabled(viewModel.isSubmitting)

                if let info = viewModel.infoMessage {
                    Text(info)
                        .font(.system(size: 12))
                        .foregroundColor(QervonColor.success)
                }
                if let error = viewModel.errorMessage {
                    Text(error)
                        .font(.system(size: 12))
                        .foregroundColor(QervonColor.danger)
                }
            }
        }
        .padding(.horizontal, QervonSpacing.lg)
    }

    private var threadSection: some View {
        VStack(alignment: .leading, spacing: QervonSpacing.sm) {
            Text("TALEP GEÇMİŞİ")
                .font(.system(size: 11, weight: .bold))
                .foregroundColor(QervonColor.textSecondary)
                .padding(.horizontal, QervonSpacing.lg)

            if viewModel.tickets.isEmpty && !viewModel.isLoading {
                QervonCard {
                    Text("Henüz destek talebiniz yok.")
                        .font(.system(size: 13))
                        .foregroundColor(QervonColor.textSecondary)
                }
                .padding(.horizontal, QervonSpacing.lg)
            } else {
                ForEach(viewModel.tickets) { ticket in
                    QervonCard {
                        VStack(alignment: .leading, spacing: QervonSpacing.xs) {
                            HStack {
                                Text(ticket.subject)
                                    .font(.system(size: 13, weight: .semibold))
                                    .foregroundColor(QervonColor.textPrimary)
                                Spacer()
                                Text(ticket.status.displayName)
                                    .font(.system(size: 11, weight: .semibold))
                                    .foregroundColor(statusColor(ticket.status))
                            }
                            Text(ticket.message)
                                .font(.system(size: 12))
                                .foregroundColor(QervonColor.textSecondary)
                            Text(QervonFormat.dayAndTime(ticket.createdAt))
                                .font(.system(size: 11))
                                .foregroundColor(QervonColor.textSecondary)
                        }
                    }
                    .padding(.horizontal, QervonSpacing.lg)
                }
            }
        }
    }

    private func statusColor(_ status: TicketStatus) -> Color {
        switch status {
        case .open: return QervonColor.warning
        case .inProgress: return QervonColor.accent
        case .resolved: return QervonColor.success
        case .closed: return QervonColor.textSecondary
        }
    }
}
