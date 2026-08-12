// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/AuthFeature/LoginView.swift
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
import UIKit
import QervonCore
import QervonNetworking
import QervonDesignSystem

public struct LoginView: View {
    @StateObject private var viewModel: AuthViewModel
    private let appTitle: String
    private let appSubtitle: String
    private let showsRegistration: Bool
    private let api: QervonAPI
    @State private var showingRegister = false

    public init(
        api: QervonAPI,
        appTitle: String = "QERVON KURYE",
        appSubtitle: String = "Lojistik İşletim Sistemi",
        showsRegistration: Bool = false,
        onLoginSucceeded: @escaping (AuthTokens) -> Void
    ) {
        self.api = api
        self.appTitle = appTitle
        self.appSubtitle = appSubtitle
        self.showsRegistration = showsRegistration
        _viewModel = StateObject(wrappedValue: AuthViewModel(api: api, onLoginSucceeded: onLoginSucceeded))
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.lg) {
                VStack(spacing: QervonSpacing.xs) {
                    Text(appTitle)
                        .font(.system(size: 26, weight: .bold))
                        .foregroundColor(QervonColor.textPrimary)
                    Text(appSubtitle)
                        .font(.system(size: 13, weight: .semibold))
                        .foregroundColor(QervonColor.accent)
                }
                .padding(.top, QervonSpacing.xl)

                Picker("Giriş Yöntemi", selection: $viewModel.mode) {
                    ForEach(LoginMode.allCases, id: \.self) { mode in
                        Text(mode.rawValue).tag(mode)
                    }
                }
                .pickerStyle(.segmented)
                .padding(.horizontal, QervonSpacing.lg)

                QervonCard {
                    VStack(spacing: QervonSpacing.md) {
                        QervonTextField(title: "Firma Kodu", text: $viewModel.tenantSlug, autocapitalize: false)

                        switch viewModel.mode {
                        case .password:
                            passwordForm
                        case .otp:
                            otpForm
                        }

                        if let errorMessage = viewModel.errorMessage {
                            Text(errorMessage)
                                .font(.system(size: 13))
                                .foregroundColor(QervonColor.danger)
                                .multilineTextAlignment(.leading)
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
                .padding(.horizontal, QervonSpacing.lg)

                if viewModel.isLoading {
                    ProgressView().tint(QervonColor.accent)
                }

                if showsRegistration {
                    Button("Hesabın yok mu? Kayıt Ol") {
                        showingRegister = true
                    }
                    .font(.system(size: 13, weight: .semibold))
                    .foregroundColor(QervonColor.textSecondary)
                }
            }
            .padding(.bottom, QervonSpacing.xl)
        }
        .qervonScreenBackground()
        .onChange(of: viewModel.mode) { _ in
            viewModel.resetOtpFlow()
        }
        .sheet(isPresented: $showingRegister) {
            RegisterView(api: api) { tokens in
                showingRegister = false
                viewModel.completeExternalLogin(tokens)
            }
        }
    }

    private var passwordForm: some View {
        VStack(spacing: QervonSpacing.md) {
            QervonTextField(title: "E-posta", text: $viewModel.email, autocapitalize: false, keyboard: .emailAddress)
            QervonTextField(title: "Parola", text: $viewModel.password, isSecure: true)
            Button("Giriş Yap") {
                Task { await viewModel.submitPasswordLogin() }
            }
            .buttonStyle(QervonButtonStyle(isEnabled: viewModel.canSubmitPassword))
            .disabled(!viewModel.canSubmitPassword || viewModel.isLoading)
        }
    }

    private var otpForm: some View {
        VStack(spacing: QervonSpacing.md) {
            switch viewModel.otpStage {
            case .enterPhone:
                QervonTextField(title: "Telefon Numarası", text: $viewModel.phone, keyboard: .phonePad)
                Button("Kod Gönder") {
                    Task { await viewModel.requestOtp() }
                }
                .buttonStyle(QervonButtonStyle(isEnabled: viewModel.canRequestOtp))
                .disabled(!viewModel.canRequestOtp || viewModel.isLoading)
            case let .enterCode(devCode):
                if let devCode {
                    Text("Geliştirme modu kodu: \(devCode)")
                        .font(.system(size: 12, weight: .semibold, design: .monospaced))
                        .foregroundColor(QervonColor.warning)
                }
                QervonTextField(title: "6 Haneli Kod", text: $viewModel.otpCode, keyboard: .numberPad)
                Button("Doğrula") {
                    Task { await viewModel.verifyOtp() }
                }
                .buttonStyle(QervonButtonStyle(isEnabled: viewModel.canVerifyOtp))
                .disabled(!viewModel.canVerifyOtp || viewModel.isLoading)
                Button("Farklı numara kullan") {
                    viewModel.resetOtpFlow()
                }
                .font(.system(size: 13, weight: .semibold))
                .foregroundColor(QervonColor.textSecondary)
            }
        }
    }
}

