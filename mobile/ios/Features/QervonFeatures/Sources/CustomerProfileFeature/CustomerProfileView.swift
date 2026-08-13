// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/CustomerProfileFeature/CustomerProfileView.swift
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
import AddressBookFeature

public struct CustomerProfileView: View {
    @StateObject private var viewModel: CustomerProfileViewModel
    @State private var supportSubject = ""
    @State private var supportMessage = ""
    private let api: QervonAPI
    let onLogout: () -> Void

    public init(api: QervonAPI, onLogout: @escaping () -> Void) {
        self.api = api
        self.onLogout = onLogout
        _viewModel = StateObject(wrappedValue: CustomerProfileViewModel(api: api))
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.lg) {
                Text("Cüzdan")
                    .font(.system(size: 18, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
                    .padding(.top, QervonSpacing.lg)

                if let profile = viewModel.profile {
                    QervonCard {
                        VStack(alignment: .leading, spacing: QervonSpacing.xs) {
                            Text("Sadakat Puanı")
                                .font(.system(size: 11, weight: .semibold))
                                .foregroundColor(QervonColor.textSecondary)
                            Text("\(profile.loyaltyPoints)")
                                .font(.system(size: 22, weight: .bold))
                                .foregroundColor(QervonColor.success)
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .padding(.horizontal, QervonSpacing.lg)
                }

                NavigationLink {
                    AddressBookListView(api: api)
                } label: {
                    navRow(title: "Adres Defteri", systemImage: "map.fill")
                }
                .padding(.horizontal, QervonSpacing.lg)

                phoneSection
                biometricSection

                Button("Çıkış Yap") {
                    onLogout()
                }
                .buttonStyle(QervonButtonStyle(kind: .destructive))
                .padding(.horizontal, QervonSpacing.lg)
            }
            .padding(.bottom, QervonSpacing.xl)
        }
        .qervonScreenBackground()
        .navigationTitle("Cüzdan")
        .task { await viewModel.load() }
        .onAppear { viewModel.startLiveSupport() }
        .onDisappear { viewModel.stopLiveSupport() }
    }

    private func navRow(title: String, systemImage: String) -> some View {
        QervonCard {
            HStack {
                Image(systemName: systemImage).foregroundColor(QervonColor.accent)
                Text(title)
                    .font(.system(size: 14, weight: .semibold))
                    .foregroundColor(QervonColor.textPrimary)
                Spacer()
                Image(systemName: "chevron.right").foregroundColor(QervonColor.textSecondary)
            }
        }
    }

    private var supportSection: some View {
        VStack(alignment: .leading, spacing: QervonSpacing.sm) {
            Text("DESTEK TALEPLERİM")
                .font(.system(size: 11, weight: .bold))
                .foregroundColor(QervonColor.textSecondary)
                .padding(.horizontal, QervonSpacing.lg)
            if viewModel.supportTickets.isEmpty {
                QervonCard {
                    Text("Açık destek talebiniz yok.")
                        .font(.system(size: 13))
                        .foregroundColor(QervonColor.textSecondary)
                }
                .padding(.horizontal, QervonSpacing.lg)
            } else {
                ForEach(viewModel.supportTickets) { ticket in
                    QervonCard {
                        VStack(alignment: .leading, spacing: QervonSpacing.xs) {
                            HStack {
                                VStack(alignment: .leading, spacing: 2) {
                                    Text(ticket.subject)
                                        .font(.system(size: 13, weight: .semibold))
                                        .foregroundColor(QervonColor.textPrimary)
                                    Text(QervonFormat.dayAndTime(ticket.createdAt))
                                        .font(.system(size: 11))
                                        .foregroundColor(QervonColor.textSecondary)
                                }
                                Spacer()
                                Text(ticket.status.displayName)
                                    .font(.system(size: 11, weight: .semibold))
                                    .foregroundColor(QervonColor.accent)
                            }
                            Text(ticket.message)
                                .font(.system(size: 12))
                                .foregroundColor(QervonColor.textSecondary)
                        }
                    }
                    .padding(.horizontal, QervonSpacing.lg)
                }
            }

            QervonCard {
                VStack(alignment: .leading, spacing: QervonSpacing.sm) {
                    QervonTextField(title: "Konu", text: $supportSubject)
                    QervonTextField(title: "Mesaj", text: $supportMessage)
                    Button("Canli Destek Talebi Olustur") {
                        Task {
                            await viewModel.submitSupportTicket(subject: supportSubject, message: supportMessage)
                        }
                    }
                    .buttonStyle(
                        QervonButtonStyle(
                            kind: .secondary,
                            isEnabled: !viewModel.isSubmittingSupportTicket &&
                                !supportSubject.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty &&
                                !supportMessage.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                        )
                    )
                    .disabled(
                        viewModel.isSubmittingSupportTicket ||
                            supportSubject.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty ||
                            supportMessage.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty
                    )
                    if let info = viewModel.supportInfoMessage {
                        Text(info)
                            .font(.system(size: 12))
                            .foregroundColor(QervonColor.success)
                    }
                }
            }
            .padding(.horizontal, QervonSpacing.lg)
        }
    }

    private var notificationsSection: some View {
        VStack(alignment: .leading, spacing: QervonSpacing.sm) {
            Text("BİLDİRİMLER")
                .font(.system(size: 11, weight: .bold))
                .foregroundColor(QervonColor.textSecondary)
                .padding(.horizontal, QervonSpacing.lg)
            if viewModel.notifications.isEmpty {
                QervonCard {
                    Text("Henüz bildiriminiz yok.")
                        .font(.system(size: 13))
                        .foregroundColor(QervonColor.textSecondary)
                }
                .padding(.horizontal, QervonSpacing.lg)
            } else {
                ForEach(viewModel.notifications) { notification in
                    QervonCard {
                        VStack(alignment: .leading, spacing: 2) {
                            Text(notification.title)
                                .font(.system(size: 13, weight: .semibold))
                                .foregroundColor(QervonColor.textPrimary)
                            Text(notification.body)
                                .font(.system(size: 12))
                                .foregroundColor(QervonColor.textSecondary)
                        }
                        .frame(maxWidth: .infinity, alignment: .leading)
                    }
                    .padding(.horizontal, QervonSpacing.lg)
                }
            }
        }
    }

    private var phoneSection: some View {
        QervonCard {
            VStack(alignment: .leading, spacing: QervonSpacing.sm) {
                Text("Telefon ile Hızlı Giriş")
                    .font(.system(size: 13, weight: .bold))
                    .foregroundColor(QervonColor.textPrimary)
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
                Text("Sipariş güncellemelerini uygulama açıkken görürsünüz. Anlık bildirim için Apple kimlik bilgisi gerektiğinden bu ortamda gerçek bildirim gönderimi devre dışıdır.")
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
                ServerAddressField()
            }
        }
        .padding(.horizontal, QervonSpacing.lg)
    }
}

private struct ServerAddressField: View {
    @State private var serverOverride: String = APIEnvironment.currentOverride() ?? ""

    var body: some View {
        VStack(alignment: .leading, spacing: QervonSpacing.sm) {
            Text("Simülatörde varsayılan olarak \(APIEnvironment.defaultBaseURL.absoluteString) kullanılır.")
                .font(.system(size: 12))
                .foregroundColor(QervonColor.textSecondary)
            QervonTextField(title: "http://192.168.x.x:8080", text: $serverOverride, autocapitalize: false)
            Button("Kaydet") {
                APIEnvironment.setOverride(serverOverride)
            }
            .buttonStyle(QervonButtonStyle(kind: .secondary))
        }
    }
}
