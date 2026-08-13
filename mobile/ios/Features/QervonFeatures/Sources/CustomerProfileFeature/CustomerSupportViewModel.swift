// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/CustomerProfileFeature/CustomerSupportViewModel.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-13
// Version:        0.1.0
//
// Description:
//   Dedicated support-center state for customer app: list/create tickets with
//   periodic live refresh.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import QervonCore
import QervonNetworking

@MainActor
public final class CustomerSupportViewModel: ObservableObject {
    @Published public private(set) var tickets: [SupportTicket] = []
    @Published public private(set) var isLoading = false
    @Published public private(set) var isSubmitting = false
    @Published public private(set) var infoMessage: String?
    @Published public private(set) var errorMessage: String?

    private let api: QervonAPI
    private var pollingTask: Task<Void, Never>?
    private let pollingInterval: Duration = .seconds(5)

    public init(api: QervonAPI) {
        self.api = api
    }

    public func load() async {
        isLoading = true
        errorMessage = nil
        defer { isLoading = false }
        do {
            tickets = try await api.listSupportTickets()
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    public func startLiveUpdates() {
        pollingTask?.cancel()
        pollingTask = Task { [weak self] in
            guard let self else { return }
            while !Task.isCancelled {
                try? await Task.sleep(for: pollingInterval)
                await refreshSilently()
            }
        }
    }

    public func stopLiveUpdates() {
        pollingTask?.cancel()
        pollingTask = nil
    }

    public func submitTicket(subject: String, message: String) async {
        let trimmedSubject = subject.trimmingCharacters(in: .whitespacesAndNewlines)
        let trimmedMessage = message.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmedSubject.isEmpty, !trimmedMessage.isEmpty else { return }

        isSubmitting = true
        infoMessage = nil
        errorMessage = nil
        defer { isSubmitting = false }
        do {
            _ = try await api.createSupportTicket(orderId: nil, subject: trimmedSubject, message: trimmedMessage)
            tickets = try await api.listSupportTickets()
            infoMessage = "Destek talebiniz iletildi."
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    private func refreshSilently() async {
        do {
            tickets = try await api.listSupportTickets()
        } catch {
            // keep previous rendered state on transient polling failures
        }
    }

    deinit {
        pollingTask?.cancel()
    }
}
