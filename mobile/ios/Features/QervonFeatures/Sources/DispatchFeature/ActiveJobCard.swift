// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/DispatchFeature/ActiveJobCard.swift
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

struct ActiveJobCard: View {
    let order: Order
    let onNavigate: () -> Void

    var body: some View {
        QervonCard(accentBorder: QervonColor.success) {
            VStack(alignment: .leading, spacing: QervonSpacing.sm) {
                HStack {
                    Text(order.status == .courierAssigned ? "ALIMA GİDİLİYOR" : "TESLİME GİDİLİYOR")
                        .font(.system(size: 11, weight: .bold))
                        .foregroundColor(QervonColor.success)
                    Spacer()
                    Text(order.status.displayName)
                        .font(.system(size: 11, weight: .semibold))
                        .foregroundColor(QervonColor.textSecondary)
                }

                Text(order.status == .courierAssigned
                     ? (order.pickup.label ?? "Alım noktası")
                     : (order.dropoff.label ?? "Teslim noktası"))
                    .font(.system(size: 16, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)

                Text(order.fare.formatted)
                    .font(.system(size: 14, weight: .bold))
                    .foregroundColor(QervonColor.success)

                Button("Navigasyona Başla", action: onNavigate)
                    .buttonStyle(QervonButtonStyle(kind: .primary))
            }
        }
        .padding(.horizontal, QervonSpacing.lg)
    }
}
