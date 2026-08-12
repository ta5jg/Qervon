// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/AuthFeature/RegisterView.swift
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

public struct RegisterView: View {
    @StateObject private var viewModel: RegisterViewModel
    @Environment(\.dismiss) private var dismiss

    public init(api: QervonAPI, onRegisteredAndLoggedIn: @escaping (AuthTokens) -> Void) {
        _viewModel = StateObject(wrappedValue: RegisterViewModel(
            api: api, onRegisteredAndLoggedIn: onRegisteredAndLoggedIn
        ))
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.lg) {
                Text("Hesap Oluştur")
                    .font(.system(size: 22, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
                    .padding(.top, QervonSpacing.xl)

                QervonCard {
                    VStack(spacing: QervonSpacing.md) {
                        QervonTextField(title: "Ad Soyad", text: $viewModel.displayName)
                        QervonTextField(
                            title: "E-posta", text: $viewModel.email,
                            autocapitalize: false, keyboard: .emailAddress
                        )
                        QervonTextField(title: "Parola (en az 12 karakter)", text: $viewModel.password, isSecure: true)
                        QervonTextField(title: "Firma Kodu", text: $viewModel.tenantSlug, autocapitalize: false)
                        Text("Firma kodu, hizmet aldığınız işletmenin size verdiği koddur.")
                            .font(.system(size: 12))
                            .foregroundColor(QervonColor.textSecondary)
                            .frame(maxWidth: .infinity, alignment: .leading)

                        if let errorMessage = viewModel.errorMessage {
                            Text(errorMessage)
                                .font(.system(size: 13))
                                .foregroundColor(QervonColor.danger)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }

                        Button("Kayıt Ol") {
                            Task { await viewModel.submit() }
                        }
                        .buttonStyle(QervonButtonStyle(isEnabled: viewModel.canSubmit))
                        .disabled(!viewModel.canSubmit || viewModel.isLoading)

                        if viewModel.isLoading {
                            ProgressView().tint(QervonColor.accent)
                        }
                    }
                }
                .padding(.horizontal, QervonSpacing.lg)

                Button("Zaten hesabın var mı? Giriş yap") {
                    dismiss()
                }
                .font(.system(size: 13, weight: .semibold))
                .foregroundColor(QervonColor.textSecondary)
            }
            .padding(.bottom, QervonSpacing.xl)
        }
        .qervonScreenBackground()
    }
}
