// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/OrdersFeature/CourierOrdersViewModel.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-13
// Version:        0.1.0
//
// Description:
//   Live active courier orders list state for the dedicated "İşlerim" tab.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import QervonCore
import QervonNetworking

@MainActor
public final class CourierOrdersViewModel: ObservableObject {
    @Published public private(set) var orders: [Order] = []
    @Published public private(set) var isLoading = false
    @Published public var errorMessage: String?

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
            orders = try await activeCourierOrders()
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

    private func activeCourierOrders() async throws -> [Order] {
        try await api.listCourierOrders().filter {
            $0.status == .courierAssigned || $0.status == .inTransit
        }
    }

    private func refreshSilently() async {
        do {
            orders = try await activeCourierOrders()
        } catch {
            // Keep previous list on transient errors.
        }
    }

    deinit {
        pollingTask?.cancel()
    }
}
