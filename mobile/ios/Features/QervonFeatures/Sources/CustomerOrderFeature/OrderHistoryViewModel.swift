// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/CustomerOrderFeature/OrderHistoryViewModel.swift
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
import QervonCore
import QervonNetworking

@MainActor
public final class OrderHistoryViewModel: ObservableObject {
    @Published public private(set) var orders: [Order] = []
    @Published public private(set) var isLoading = false
    @Published public var errorMessage: String?

    private let api: QervonAPI

    public init(api: QervonAPI) {
        self.api = api
    }

    public func load() async {
        isLoading = true
        errorMessage = nil
        defer { isLoading = false }
        do {
            orders = try await api.listCustomerOrders().sorted { $0.createdAt > $1.createdAt }
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    public var activeOrders: [Order] {
        orders.filter {
            $0.status == .pending || $0.status == .courierAssigned || $0.status == .inTransit
        }
    }

    public var pastOrders: [Order] {
        orders.filter {
            $0.status == .delivered || $0.status == .cancelled || $0.status == .returned
        }
    }
}
