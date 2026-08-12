// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/CustomerOrderFeature/OrderDetailViewModel.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Drives the order detail/tracking screen: polls the courier's last
//   reported location and the ETA endpoint (~5s) while the order is
//   active, and exposes cancel/rate actions. Polling, not a WebSocket
//   subscription — consistent with the Courier app's job-offer polling and
//   much simpler for a single-order view.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import QervonCore
import QervonNetworking

@MainActor
public final class OrderDetailViewModel: ObservableObject {
    @Published public private(set) var order: Order
    @Published public private(set) var courierLocation: LocationSnapshot?
    @Published public private(set) var eta: EtaInfo?
    @Published public private(set) var isCancelling = false
    @Published public var errorMessage: String?

    private let api: QervonAPI
    private var pollingTask: Task<Void, Never>?
    private let pollingInterval: Duration = .seconds(5)

    public init(order: Order, api: QervonAPI) {
        self.order = order
        self.api = api
    }

    public var isTrackable: Bool {
        order.status == .courierAssigned || order.status == .inTransit
    }

    public var canCancel: Bool {
        order.status == .pending || order.status == .courierAssigned
    }

    public func onAppear() {
        startPolling()
    }

    public func onDisappear() {
        pollingTask?.cancel()
    }

    private func startPolling() {
        pollingTask?.cancel()
        pollingTask = Task { [weak self] in
            guard let self else { return }
            while !Task.isCancelled {
                await self.refresh()
                if !self.isTrackable {
                    return
                }
                try? await Task.sleep(for: self.pollingInterval)
            }
        }
    }

    private func refresh() async {
        guard isTrackable else { return }
        async let locationResult = try? api.getOrderTracking(orderId: order.id)
        async let etaResult = try? api.getOrderEta(orderId: order.id)
        courierLocation = await locationResult ?? nil
        eta = await etaResult ?? nil
    }

    public func cancel() async -> Bool {
        isCancelling = true
        errorMessage = nil
        defer { isCancelling = false }
        do {
            order = try await api.cancelOrder(orderId: order.id)
            pollingTask?.cancel()
            return true
        } catch {
            errorMessage = error.localizedDescription
            return false
        }
    }

    public func rate(stars: Int, comment: String?) async -> Bool {
        errorMessage = nil
        do {
            _ = try await api.rateOrder(orderId: order.id, stars: stars, comment: comment)
            return true
        } catch {
            errorMessage = error.localizedDescription
            return false
        }
    }

    public func openSupportTicket(subject: String, message: String) async -> Bool {
        errorMessage = nil
        do {
            _ = try await api.createSupportTicket(orderId: order.id, subject: subject, message: message)
            return true
        } catch {
            errorMessage = error.localizedDescription
            return false
        }
    }
}
