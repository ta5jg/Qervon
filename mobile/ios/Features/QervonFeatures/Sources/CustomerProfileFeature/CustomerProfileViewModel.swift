// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/CustomerProfileFeature/CustomerProfileViewModel.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import UIKit
import UserNotifications
import QervonCore
import QervonNetworking
import QervonSecurity

@MainActor
public final class CustomerProfileViewModel: ObservableObject {
    @Published public private(set) var profile: CustomerProfile?
    @Published public var phoneInput = ""
    @Published public private(set) var isLinkingPhone = false
    @Published public private(set) var phoneLinked = false
    @Published public private(set) var supportTickets: [SupportTicket] = []
    @Published public private(set) var isSubmittingSupportTicket = false
    @Published public private(set) var supportInfoMessage: String?
    @Published public private(set) var notifications: [AppNotification] = []
    @Published public var errorMessage: String?
    @Published public var isBiometricEnabled: Bool {
        didSet { AppPreferences.shared.isBiometricUnlockEnabled = isBiometricEnabled }
    }

    public let biometricGate = BiometricGate()
    private let api: QervonAPI
    private var supportPollingTask: Task<Void, Never>?
    private let supportPollingInterval: Duration = .seconds(5)

    public init(api: QervonAPI) {
        self.api = api
        self.isBiometricEnabled = AppPreferences.shared.isBiometricUnlockEnabled
    }

    public func load() async {
        async let profileResult = try? api.getCustomerProfile()
        async let ticketsResult = try? api.listSupportTickets()
        async let notificationsResult = try? api.listNotifications()
        profile = await profileResult
        supportTickets = await ticketsResult ?? []
        notifications = await notificationsResult ?? []
    }

    public func startLiveSupport() {
        supportPollingTask?.cancel()
        supportPollingTask = Task { [weak self] in
            guard let self else { return }
            while !Task.isCancelled {
                try? await Task.sleep(for: supportPollingInterval)
                await refreshSupportTickets()
            }
        }
    }

    public func stopLiveSupport() {
        supportPollingTask?.cancel()
        supportPollingTask = nil
    }

    public func submitSupportTicket(subject: String, message: String) async {
        let trimmedSubject = subject.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedMessage = message.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedSubject.isEmpty, !trimmedMessage.isEmpty else { return }

        isSubmittingSupportTicket = true
        supportInfoMessage = nil
        errorMessage = nil
        defer { isSubmittingSupportTicket = false }

        do {
            _ = try await api.createSupportTicket(orderId: nil, subject: trimmedSubject, message: trimmedMessage)
            supportTickets = try await api.listSupportTickets()
            supportInfoMessage = "Destek talebiniz operatore iletildi."
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    private func refreshSupportTickets() async {
        do {
            supportTickets = try await api.listSupportTickets()
        } catch {
            // Keep last rendered support list during transient failures.
        }
    }

    public func linkPhone() async {
        guard !phoneInput.trimmingCharacters(in: .whitespaces).isEmpty else { return }
        isLinkingPhone = true
        errorMessage = nil
        defer { isLinkingPhone = false }
        do {
            _ = try await api.linkPhone(phoneInput)
            phoneLinked = true
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    public func requestPushPermission() async -> Bool {
        do {
            let granted = try await UNUserNotificationCenter.current()
                .requestAuthorization(options: [.alert, .sound, .badge])
            if granted {
                UIApplication.shared.registerForRemoteNotifications()
            }
            return granted
        } catch {
            errorMessage = error.localizedDescription
            return false
        }
    }

    deinit {
        supportPollingTask?.cancel()
    }
}
