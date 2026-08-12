// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/CustomerOrderFeature/RatingSheet.swift
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
import QervonDesignSystem

struct RatingSheet: View {
    let onSubmit: (Int, String?) async -> Bool
    @State private var stars = 5
    @State private var comment = ""
    @State private var isSubmitting = false
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(spacing: QervonSpacing.lg) {
            Text("Teslimatı Değerlendir")
                .font(.system(size: 18, weight: .bold))
                .foregroundColor(QervonColor.textPrimary)
                .padding(.top, QervonSpacing.lg)

            HStack(spacing: QervonSpacing.sm) {
                ForEach(1...5, id: \.self) { value in
                    Image(systemName: value <= stars ? "star.fill" : "star")
                        .font(.system(size: 28))
                        .foregroundColor(QervonColor.warning)
                        .onTapGesture { stars = value }
                }
            }

            QervonTextField(title: "Yorumunuz (opsiyonel)", text: $comment)
                .padding(.horizontal, QervonSpacing.lg)

            Button("Gönder") {
                Task {
                    isSubmitting = true
                    let success = await onSubmit(stars, comment.isEmpty ? nil : comment)
                    isSubmitting = false
                    if success { dismiss() }
                }
            }
            .buttonStyle(QervonButtonStyle(isEnabled: !isSubmitting))
            .disabled(isSubmitting)
            .padding(.horizontal, QervonSpacing.lg)

            Spacer()
        }
        .qervonScreenBackground()
        .presentationDetents([.medium])
    }
}
