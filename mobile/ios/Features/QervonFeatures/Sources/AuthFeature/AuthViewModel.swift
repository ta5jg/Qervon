// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/AuthFeature/AuthViewModel.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Drives the login screen: password login (always available) and
//   phone+OTP login (only useful once a phone has been linked to the
//   account via the Profile screen, since OTP cannot create an account by
//   itself — see backend/crates/application/src/otp_service.rs).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import QervonCore
import QervonNetworking
import QervonSecurity

public enum LoginMode: String, CaseIterable {
    case password = "Parola"
    case otp = "Telefon / OTP"
}

public enum OtpStage: Equatable {
    case enterPhone
    case enterCode(devCode: String?)
}

@MainActor
public final class AuthViewModel: ObservableObject {
    @Published public var mode: LoginMode = .password
    @Published public var tenantSlug: String
    @Published public var email = ""
    @Published public var password = ""
    @Published public var phone = ""
    @Published public var otpCode = ""
    @Published public var otpStage: OtpStage = .enterPhone
    @Published public private(set) var isLoading = false
    @Published public var errorMessage: String?

    private let api: QervonAPI
    private let onLoginSucceeded: (AuthTokens) -> Void

    public init(api: QervonAPI, onLoginSucceeded: @escaping (AuthTokens) -> Void) {
        self.api = api
        self.onLoginSucceeded = onLoginSucceeded
        self.tenantSlug = AppPreferences.shared.lastTenantSlug ?? ""
    }

    public var canSubmitPassword: Bool {
        !tenantSlug.trimmingCharacters(in: .whitespaces).isEmpty
            && !email.trimmingCharacters(in: .whitespaces).isEmpty
            && password.count >= 12
    }

    public func submitPasswordLogin() async {
        guard canSubmitPassword else { return }
        await run {
            let tokens = try await api.login(email: email, password: password, tenantSlug: tenantSlug)
            AppPreferences.shared.lastTenantSlug = tenantSlug
            onLoginSucceeded(tokens)
        }
    }

    public var canRequestOtp: Bool {
        !tenantSlug.trimmingCharacters(in: .whitespaces).isEmpty
            && !phone.trimmingCharacters(in: .whitespaces).isEmpty
    }

    public func requestOtp() async {
        guard canRequestOtp else { return }
        await run {
            let result = try await api.requestOtp(tenantSlug: tenantSlug, phone: phone)
            AppPreferences.shared.lastTenantSlug = tenantSlug
            otpStage = .enterCode(devCode: result.devCode)
            if let devCode = result.devCode {
                // Local/dev backend only — never present on a real deployment.
                otpCode = devCode
            }
        }
    }

    public var canVerifyOtp: Bool {
        otpCode.count == 6
    }

    public func verifyOtp() async {
        guard canVerifyOtp else { return }
        await run {
            let tokens = try await api.verifyOtp(tenantSlug: tenantSlug, phone: phone, code: otpCode)
            onLoginSucceeded(tokens)
        }
    }

    public func resetOtpFlow() {
        otpStage = .enterPhone
        otpCode = ""
        errorMessage = nil
    }

    /// Forwards tokens obtained outside this view model's own flows (e.g.
    /// the register-then-login sequence in `RegisterViewModel`) through the
    /// same success callback the login screen uses.
    public func completeExternalLogin(_ tokens: AuthTokens) {
        onLoginSucceeded(tokens)
    }

    private func run(_ operation: () async throws -> Void) async {
        isLoading = true
        errorMessage = nil
        defer { isLoading = false }
        do {
            try await operation()
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
