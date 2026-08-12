// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/EarningsFeature/EarningsViewModel.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Loads the wallet and the courier's own ratings. Period totals
//   (today/week/month) are computed client-side from the wallet's
//   transaction list — the backend has no aggregation endpoint for this.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import QervonCore
import QervonNetworking

@MainActor
public final class EarningsViewModel: ObservableObject {
    @Published public private(set) var wallet: CourierWallet?
    @Published public private(set) var ratings: [CustomerRating] = []
    @Published public private(set) var isLoading = false
    @Published public var errorMessage: String?

    private let api: QervonAPI

    public init(api: QervonAPI) {
        self.api = api
    }

    public var todayEarnings: Money? {
        wallet?.totalCredited(since: QervonFormat.startOfDay())
    }

    public var weekEarnings: Money? {
        wallet?.totalCredited(since: QervonFormat.startOfWeek())
    }

    public var monthEarnings: Money? {
        wallet?.totalCredited(since: QervonFormat.startOfMonth())
    }

    public var averageRating: Double? {
        ratings.averageStars
    }

    public func load() async {
        isLoading = true
        errorMessage = nil
        defer { isLoading = false }
        async let walletResult = api.getWallet()
        async let ratingsResult = api.getOwnRatings()
        do {
            wallet = try await walletResult
            ratings = try await ratingsResult
        } catch {
            errorMessage = error.localizedDescription
        }
    }
}
