// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/DispatchFeature/DispatchViewModel.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Drives the courier's home screen: online/offline toggle, polling for a
//   pending job offer (no push channel offers jobs today — see
//   BACKEND_BACKLOG.md / GET /v1/courier/me/offer), accept/reject, and the
//   currently active (accepted) job.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import QervonCore
import QervonNetworking
import QervonLocation

@MainActor
public final class DispatchViewModel: ObservableObject {
    @Published public private(set) var courier: Courier?
    @Published public private(set) var pendingOffer: PendingOffer?
    @Published public private(set) var activeOrder: Order?
    @Published public private(set) var isTogglingOnline = false
    @Published public private(set) var isRespondingToOffer = false
    @Published public var errorMessage: String?
    @Published public private(set) var secondsRemainingOnOffer: TimeInterval = 0

    public let locationBroadcaster: CourierLocationBroadcaster

    private let api: QervonAPI
    private var pollingTask: Task<Void, Never>?
    private var countdownTask: Task<Void, Never>?
    private let pollingInterval: Duration = .seconds(6)

    public var isOnline: Bool { courier?.status != .offline }

    public init(api: QervonAPI) {
        self.api = api
        self.locationBroadcaster = CourierLocationBroadcaster(api: api)
    }

    public func onAppear() {
        locationBroadcaster.requestPermission()
        startPolling()
    }

    public func onDisappear() {
        pollingTask?.cancel()
        countdownTask?.cancel()
    }

    public func refreshAll() async {
        async let courierResult = try? api.getOwnCourier()
        async let orders = try? api.listCourierOrders()
        courier = await courierResult
        activeOrder = await orders?.first
        if activeOrder == nil {
            await refreshPendingOffer()
        } else {
            pendingOffer = nil
        }
    }

    public func toggleOnline() async {
        isTogglingOnline = true
        defer { isTogglingOnline = false }
        do {
            let wantsOnline = !isOnline
            courier = try await api.setAvailability(online: wantsOnline)
            if wantsOnline {
                locationBroadcaster.startBroadcasting()
            } else {
                locationBroadcaster.stopBroadcasting()
            }
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    public func acceptOffer() async {
        guard let offer = pendingOffer else { return }
        isRespondingToOffer = true
        defer { isRespondingToOffer = false }
        do {
            activeOrder = try await api.acceptOffer(orderId: offer.order.id)
            pendingOffer = nil
            courier = try await api.getOwnCourier()
        } catch {
            errorMessage = error.localizedDescription
            // The offer may have just expired server-side; refresh to find out.
            await refreshPendingOffer()
        }
    }

    public func rejectOffer() async {
        guard let offer = pendingOffer else { return }
        isRespondingToOffer = true
        defer { isRespondingToOffer = false }
        do {
            try await api.rejectOffer(orderId: offer.order.id)
            pendingOffer = nil
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    /// Called by the delivery flow once an order reaches a terminal state,
    /// so the home screen resumes polling for the next offer immediately.
    public func clearActiveOrder() {
        activeOrder = nil
    }

    private func refreshPendingOffer() async {
        do {
            let offer = try await api.getPendingOffer()
            pendingOffer = offer
            startCountdown(for: offer)
        } catch {
            // A transient polling failure should not surface as a hard
            // error banner; the next poll will simply retry.
        }
    }

    private func startPolling() {
        pollingTask?.cancel()
        pollingTask = Task { [weak self] in
            guard let self else { return }
            while !Task.isCancelled {
                await self.refreshAll()
                try? await Task.sleep(for: self.pollingInterval)
            }
        }
    }

    private func startCountdown(for offer: PendingOffer?) {
        countdownTask?.cancel()
        guard let offer else {
            secondsRemainingOnOffer = 0
            return
        }
        countdownTask = Task { [weak self] in
            while !Task.isCancelled {
                let remaining = offer.secondsRemaining()
                self?.secondsRemainingOnOffer = remaining
                if remaining <= 0 {
                    self?.pendingOffer = nil
                    return
                }
                try? await Task.sleep(for: .seconds(1))
            }
        }
    }
}
