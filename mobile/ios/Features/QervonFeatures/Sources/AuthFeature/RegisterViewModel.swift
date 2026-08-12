// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/AuthFeature/RegisterViewModel.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Drives account registration (`POST /v1/auth/register`). The endpoint
//   creates a `role=customer` account and, when a tenant code is given,
//   joins that tenant — but it never returns tokens, so a successful
//   registration is immediately followed by a real password login using
//   the same credentials.
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
    @Published public var password = ""
    @Published public var tenantSlug: String
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
            && password.count >= 12
    }

    public func submit() async {
        guard canSubmit else { return }
        isLoading = true
        errorMessage = nil
        defer { isLoading = false }
        do {
            try await api.register(
                email: email,
                displayName: displayName,
                password: password,
                tenantSlug: tenantSlug
            )
            AppPreferences.shared.lastTenantSlug = tenantSlug
            let tokens = try await api.login(email: email, password: password, tenantSlug: tenantSlug)
            onRegisteredAndLoggedIn(tokens)
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
