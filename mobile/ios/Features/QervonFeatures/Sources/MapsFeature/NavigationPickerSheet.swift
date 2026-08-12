// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/MapsFeature/NavigationPickerSheet.swift
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

/// A bottom sheet listing every navigation app currently installed (Apple
/// Maps is always listed) so the courier can pick where to open directions.
public struct NavigationPickerSheet: View {
    let destination: GeoLocation
    let label: String
    @Environment(\.dismiss) private var dismiss

    public init(destination: GeoLocation, label: String) {
        self.destination = destination
        self.label = label
    }

    public var body: some View {
        VStack(spacing: QervonSpacing.md) {
            Text("Navigasyona Başla")
                .font(.system(size: 17, weight: .bold))
                .foregroundColor(QervonColor.textPrimary)
            Text(label)
                .font(.system(size: 13))
                .foregroundColor(QervonColor.textSecondary)

            ForEach(NavigationApp.allCases.filter { $0.isAvailable(for: destination) }) { app in
                Button(app.rawValue) {
                    NavigationLauncher.open(app, to: destination)
                    dismiss()
                }
                .buttonStyle(QervonButtonStyle(kind: .secondary))
            }
        }
        .padding(QervonSpacing.lg)
        .qervonScreenBackground()
        .presentationDetents([.medium])
    }
}
