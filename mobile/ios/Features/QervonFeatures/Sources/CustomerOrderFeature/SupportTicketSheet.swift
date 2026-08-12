// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/CustomerOrderFeature/SupportTicketSheet.swift
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

struct SupportTicketSheet: View {
    let onSubmit: (String, String) async -> Bool
    @State private var subject = ""
    @State private var message = ""
    @State private var isSubmitting = false
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(spacing: QervonSpacing.lg) {
            Text("Destek Talebi Aç")
                .font(.system(size: 18, weight: .bold))
                .foregroundColor(QervonColor.textPrimary)
                .padding(.top, QervonSpacing.lg)

            QervonTextField(title: "Konu", text: $subject)
                .padding(.horizontal, QervonSpacing.lg)
            QervonTextField(title: "Mesajınız", text: $message)
                .padding(.horizontal, QervonSpacing.lg)

            Button("Gönder") {
                Task {
                    isSubmitting = true
                    let success = await onSubmit(subject, message)
                    isSubmitting = false
                    if success { dismiss() }
                }
            }
            .buttonStyle(QervonButtonStyle(
                isEnabled: !subject.trimmingCharacters(in: .whitespaces).isEmpty
                    && !message.trimmingCharacters(in: .whitespaces).isEmpty && !isSubmitting
            ))
            .disabled(
                subject.trimmingCharacters(in: .whitespaces).isEmpty
                    || message.trimmingCharacters(in: .whitespaces).isEmpty || isSubmitting
            )
            .padding(.horizontal, QervonSpacing.lg)

            Spacer()
        }
        .qervonScreenBackground()
        .presentationDetents([.medium])
    }
}
