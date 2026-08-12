// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/Models/Order.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `OrderResponse` (`backend/crates/api-contracts/src/lib.rs`) and
//   `qervon_domain::{OrderStatus, PaymentMethod}`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public enum OrderStatus: String, Codable, Sendable, Equatable {
    case pending
    case courierAssigned = "courier_assigned"
    case inTransit = "in_transit"
    case delivered
    case cancelled
    case returned

    public var displayName: String {
        switch self {
        case .pending: return "Bekliyor"
        case .courierAssigned: return "Kurye Atandı"
        case .inTransit: return "Yolda"
        case .delivered: return "Teslim Edildi"
        case .cancelled: return "İptal"
        case .returned: return "İade"
        }
    }
}

public enum PaymentMethod: String, Codable, Sendable, Equatable, CaseIterable {
    case cash
    case card
    case qr
    case wallet

    public var displayName: String {
        switch self {
        case .cash: return "Nakit"
        case .card: return "Kart"
        case .qr: return "QR"
        case .wallet: return "Cüzdan"
        }
    }
}

public struct Order: Codable, Sendable, Equatable, Identifiable {
    public let id: UUID
    public let customerId: UUID
    public let pickup: Address
    public let dropoff: Address
    public let status: OrderStatus
    public let fare: Money
    public let assignedCourierId: UUID?
    public let createdAt: Date
    public let deliveredAt: Date?
    public let returnedAt: Date?
    public let paymentMethod: PaymentMethod?
    public let paymentCollected: Bool
    public let deliveryNote: String?
    public let contactPhone: String?

    public init(
        id: UUID,
        customerId: UUID,
        pickup: Address,
        dropoff: Address,
        status: OrderStatus,
        fare: Money,
        assignedCourierId: UUID?,
        createdAt: Date,
        deliveredAt: Date?,
        returnedAt: Date?,
        paymentMethod: PaymentMethod?,
        paymentCollected: Bool,
        deliveryNote: String? = nil,
        contactPhone: String? = nil
    ) {
        self.id = id
        self.customerId = customerId
        self.pickup = pickup
        self.dropoff = dropoff
        self.status = status
        self.fare = fare
        self.assignedCourierId = assignedCourierId
        self.createdAt = createdAt
        self.deliveredAt = deliveredAt
        self.returnedAt = returnedAt
        self.paymentMethod = paymentMethod
        self.paymentCollected = paymentCollected
        self.deliveryNote = deliveryNote
        self.contactPhone = contactPhone
    }
}
