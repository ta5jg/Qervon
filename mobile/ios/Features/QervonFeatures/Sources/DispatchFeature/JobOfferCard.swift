// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/DispatchFeature/JobOfferCard.swift
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
import QervonDesignSystem

struct JobOfferCard: View {
    let offer: PendingOffer
    let secondsRemaining: TimeInterval
    let isResponding: Bool
    let onAccept: () -> Void
    let onReject: () -> Void

    var body: some View {
        QervonCard(accentBorder: QervonColor.accent) {
            VStack(alignment: .leading, spacing: QervonSpacing.sm) {
                HStack {
                    Text("YENİ İŞ TEKLİFİ")
                        .font(.system(size: 11, weight: .bold))
                        .foregroundColor(QervonColor.accent)
                    Spacer()
                    Text("\(Int(secondsRemaining))s")
                        .font(.system(size: 13, weight: .bold, design: .monospaced))
                        .foregroundColor(secondsRemaining <= 10 ? QervonColor.danger : QervonColor.textSecondary)
                }

                Text(offer.order.pickup.label ?? "Alım noktası")
                    .font(.system(size: 15, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
                Text("➔ \(offer.order.dropoff.label ?? "Teslim noktası")")
                    .font(.system(size: 13))
                    .foregroundColor(QervonColor.textSecondary)

                HStack {
                    Text(offer.order.fare.formatted)
                        .font(.system(size: 15, weight: .bold))
                        .foregroundColor(QervonColor.success)
                    Spacer()
                    if let method = offer.order.paymentMethod {
                        Text(method.displayName)
                            .font(.system(size: 12, weight: .semibold))
                            .foregroundColor(QervonColor.textSecondary)
                    }
                }

                HStack(spacing: QervonSpacing.sm) {
                    Button("Reddet", action: onReject)
                        .buttonStyle(QervonButtonStyle(kind: .destructive, isEnabled: !isResponding))
                        .disabled(isResponding)
                    Button("Kabul Et", action: onAccept)
                        .buttonStyle(QervonButtonStyle(kind: .primary, isEnabled: !isResponding))
                        .disabled(isResponding)
                }
            }
        }
        .padding(.horizontal, QervonSpacing.lg)
    }
}
