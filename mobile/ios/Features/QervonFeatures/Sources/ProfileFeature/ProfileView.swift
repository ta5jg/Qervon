// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/ProfileFeature/ProfileView.swift
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

public struct ProfileView: View {
    @StateObject private var viewModel: ProfileViewModel
    let onLogout: () -> Void
    @State private var serverOverride: String = APIEnvironment.currentOverride() ?? ""

    public init(api: QervonAPI, onLogout: @escaping () -> Void) {
        _viewModel = StateObject(wrappedValue: ProfileViewModel(api: api))
        self.onLogout = onLogout
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.lg) {
                Text("Profil")
                    .font(.system(size: 18, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
                    .padding(.top, QervonSpacing.lg)

                if let courier = viewModel.courier {
                    QervonCard {
                        VStack(alignment: .leading, spacing: QervonSpacing.xs) {
                            Text(courier.name)
                                .font(.system(size: 16, weight: .bold))
                                .foregroundColor(QervonColor.textPrimary)
                            Text(courier.vehicle.displayName)
                                .font(.system(size: 13))
                                .foregroundColor(QervonColor.textSecondary)
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .padding(.horizontal, QervonSpacing.lg)
                }

                phoneSection
                biometricSection
                pushSection
                serverSection

                Button("Çıkış Yap") {
                    onLogout()
                }
                .buttonStyle(QervonButtonStyle(kind: .destructive))
                .padding(.horizontal, QervonSpacing.lg)
            }
            .padding(.bottom, QervonSpacing.xl)
        }
        .qervonScreenBackground()
        .task { await viewModel.load() }
    }

    private var phoneSection: some View {
        QervonCard {
            VStack(alignment: .leading, spacing: QervonSpacing.sm) {
                Text("Telefon ile Hızlı Giriş")
                    .font(.system(size: 13, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
                Text("Telefon numaranızı bağlarsanız sonraki girişlerde OTP kullanabilirsiniz.")
                    .font(.system(size: 12))
                    .foregroundColor(QervonColor.textSecondary)
                if viewModel.phoneLinked {
                    Text("Telefon numarası bağlandı ✓")
                        .font(.system(size: 12, weight: .semibold))
                        .foregroundColor(QervonColor.success)
                } else {
                    QervonTextField(title: "Telefon Numarası", text: $viewModel.phoneInput, keyboard: .phonePad)
                    Button("Bağla") {
                        Task { await viewModel.linkPhone() }
                    }
                    .buttonStyle(QervonButtonStyle(kind: .secondary, isEnabled: !viewModel.isLinkingPhone))
                    .disabled(viewModel.isLinkingPhone)
                }
                if let errorMessage = viewModel.errorMessage {
                    Text(errorMessage)
                        .font(.system(size: 12))
                        .foregroundColor(QervonColor.danger)
                }
            }
        }
        .padding(.horizontal, QervonSpacing.lg)
    }

    private var biometricSection: some View {
        QervonCard {
            let kind = viewModel.biometricGate.availableKind()
            if kind == .none {
                Text("Bu cihazda biyometrik doğrulama kullanılamıyor.")
                    .font(.system(size: 12))
                    .foregroundColor(QervonColor.textSecondary)
            } else {
                Toggle(
                    kind == .faceID ? "Face ID ile Hızlı Giriş" : "Touch ID ile Hızlı Giriş",
                    isOn: $viewModel.isBiometricEnabled
                )
                .tint(QervonColor.accent)
                .foregroundColor(QervonColor.textPrimary)
            }
        }
        .padding(.horizontal, QervonSpacing.lg)
    }

    private var pushSection: some View {
        QervonCard {
            VStack(alignment: .leading, spacing: QervonSpacing.sm) {
                Text("Bildirimler")
                    .font(.system(size: 13, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
                Text("Yeni iş tekliflerini uygulama açıkken görürsünüz. Anlık bildirim için Apple/Google kimlik bilgisi gerektiğinden bu ortamda gerçek bildirim gönderimi devre dışıdır.")
                    .font(.system(size: 12))
                    .foregroundColor(QervonColor.textSecondary)
                Button("Bildirim İzni İste") {
                    Task { _ = await viewModel.requestPushPermission() }
                }
                .buttonStyle(QervonButtonStyle(kind: .secondary))
            }
        }
        .padding(.horizontal, QervonSpacing.lg)
    }

    private var serverSection: some View {
        QervonCard {
            VStack(alignment: .leading, spacing: QervonSpacing.sm) {
                Text("Sunucu Adresi (gelişmiş)")
                    .font(.system(size: 13, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
                Text("Simülatörde varsayılan olarak \(APIEnvironment.defaultBaseURL.absoluteString) kullanılır. Gerçek bir cihazda test ederken Mac'inizin ağ adresini girin.")
                    .font(.system(size: 12))
                    .foregroundColor(QervonColor.textSecondary)
                QervonTextField(title: "http://192.168.x.x:8080", text: $serverOverride, autocapitalize: false)
                Button("Kaydet") {
                    APIEnvironment.setOverride(serverOverride)
                }
                .buttonStyle(QervonButtonStyle(kind: .secondary))
            }
        }
        .padding(.horizontal, QervonSpacing.lg)
    }
}
