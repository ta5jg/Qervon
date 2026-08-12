// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/CustomerOrderFeature/NewOrderViewModel.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Drives the "Yeni Sipariş" screen: pickup/dropoff selection, a live
//   (non-binding) fare quote as both points are set, and submission. The
//   server always recomputes the authoritative fare at creation time —
//   the quote shown here is purely informational.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import QervonCore
import QervonNetworking

@MainActor
public final class NewOrderViewModel: ObservableObject {
    @Published public var pickup: Address?
    @Published public var dropoff: Address?
    @Published public var paymentMethod: PaymentMethod = .cash
    @Published public var couponCode = ""
    @Published public var deliveryNote = ""
    @Published public var contactPhone = ""

    @Published public private(set) var quote: FareQuote?
    @Published public private(set) var isQuoting = false
    @Published public private(set) var isSubmitting = false
    @Published public var errorMessage: String?

    private let api: QervonAPI
    private var quoteTask: Task<Void, Never>?

    public init(api: QervonAPI) {
        self.api = api
    }

    public var canSubmit: Bool {
        pickup != nil && dropoff != nil && !isSubmitting
    }

    public func setPickup(_ address: Address) {
        pickup = address
        refreshQuote()
    }

    public func setDropoff(_ address: Address) {
        dropoff = address
        refreshQuote()
    }

    public func refreshQuote() {
        quoteTask?.cancel()
        guard let pickup, let dropoff else {
            quote = nil
            return
        }
        quoteTask = Task {
            isQuoting = true
            defer { isQuoting = false }
            do {
                let result = try await api.getFareQuote(
                    pickup: GeoLocation(latitude: pickup.latitude, longitude: pickup.longitude),
                    dropoff: GeoLocation(latitude: dropoff.latitude, longitude: dropoff.longitude)
                )
                if !Task.isCancelled {
                    quote = result
                }
            } catch {
                if !Task.isCancelled {
                    errorMessage = error.localizedDescription
                }
            }
        }
    }

    public func submit() async -> Order? {
        guard let pickup, let dropoff else { return nil }
        isSubmitting = true
        errorMessage = nil
        defer { isSubmitting = false }
        let body = CreateCustomerOrderBody(
            pickup: pickup,
            dropoff: dropoff,
            couponCode: couponCode.trimmingCharacters(in: .whitespaces).isEmpty ? nil : couponCode,
            paymentMethod: paymentMethod,
            deliveryNote: deliveryNote.trimmingCharacters(in: .whitespaces).isEmpty ? nil : deliveryNote,
            contactPhone: contactPhone.trimmingCharacters(in: .whitespaces).isEmpty ? nil : contactPhone
        )
        do {
            return try await api.createOrder(body)
        } catch {
            errorMessage = error.localizedDescription
            return nil
        }
    }

    public func reset() {
        pickup = nil
        dropoff = nil
        couponCode = ""
        deliveryNote = ""
        contactPhone = ""
        quote = nil
        errorMessage = nil
    }
}
