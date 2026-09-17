// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/AuthFeature/RegisterViewModel.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Customer signup with required tenant, phone, password confirmation,
//   and email/SMS OTP before `POST /v1/auth/register`, then password login.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import QervonCore
import QervonNetworking
import QervonSecurity

@MainActor
public final class RegisterViewModel: ObservableObject {
    @Published public var displayName = ""
    @Published public var email = ""
    @Published public var phone = ""
    @Published public var password = ""
    @Published public var passwordConfirm = ""
    @Published public var tenantSlug: String
    @Published public var emailOtp = ""
    @Published public var phoneOtp = ""
    @Published public var codesSent = false
    @Published public var emailDevCode: String?
    @Published public var phoneDevCode: String?
    @Published public private(set) var isLoading = false
    @Published public var errorMessage: String?

    private let api: QervonAPI
    private let onRegisteredAndLoggedIn: (AuthTokens) -> Void

    public init(api: QervonAPI, onRegisteredAndLoggedIn: @escaping (AuthTokens) -> Void) {
        self.api = api
        self.onRegisteredAndLoggedIn = onRegisteredAndLoggedIn
        self.tenantSlug = AppPreferences.shared.lastTenantSlug ?? ""
    }

    public var canSubmit: Bool {
        !displayName.trimmingCharacters(in: .whitespaces).isEmpty
            && !email.trimmingCharacters(in: .whitespaces).isEmpty
            && !tenantSlug.trimmingCharacters(in: .whitespaces).isEmpty
            && phone.filter(\.isNumber).count >= 10
            && password.count >= 12
            && password == passwordConfirm
            && emailOtp.count == 6
            && phoneOtp.count == 6
    }

    public func requestCodes() async {
        let slug = tenantSlug.trimmingCharacters(in: .whitespaces)
        let signupEmail = email.trimmingCharacters(in: .whitespaces)
        let signupPhone = phone.trimmingCharacters(in: .whitespaces)
        guard !slug.isEmpty, !signupEmail.isEmpty, signupPhone.filter(\.isNumber).count >= 10 else {
            errorMessage = "Şirket kodu, e-posta ve geçerli bir telefon numarası gerekli."
            return
        }
        isLoading = true
        errorMessage = nil
        defer { isLoading = false }
        do {
            let emailResult = try await api.requestSignupVerification(
                tenantSlug: slug, channel: "email", destination: signupEmail
            )
            let phoneResult = try await api.requestSignupVerification(
                tenantSlug: slug, channel: "sms", destination: signupPhone
            )
            codesSent = true
            emailDevCode = emailResult.devCode
            phoneDevCode = phoneResult.devCode
            if let emailDevCode { emailOtp = emailDevCode }
            if let phoneDevCode { phoneOtp = phoneDevCode }
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    public func submit() async {
        guard canSubmit else {
            if password != passwordConfirm {
                errorMessage = "Parolalar eşleşmiyor."
            }
            return
        }
        isLoading = true
        errorMessage = nil
        defer { isLoading = false }
        do {
            try await api.register(
                email: email,
                displayName: displayName,
                password: password,
                tenantSlug: tenantSlug,
                phone: phone,
                emailOtp: emailOtp,
                phoneOtp: phoneOtp
            )
            AppPreferences.shared.lastTenantSlug = tenantSlug
            let tokens = try await api.login(email: email, password: password, tenantSlug: tenantSlug)
            onRegisteredAndLoggedIn(tokens)
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
