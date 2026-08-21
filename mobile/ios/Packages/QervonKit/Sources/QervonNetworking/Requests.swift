// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonNetworking/Requests.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Request/response body shapes specific to individual endpoints (as
//   opposed to the shared domain models in `QervonCore`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import QervonCore

struct OtpRequestBody: Encodable {
    let tenantSlug: String
    let phone: String
}

public struct OtpRequestResult: Decodable, Sendable {
    public let status: String
    /// Only populated on in-memory (local/dev) backend storage — see
    /// BACKEND_BACKLOG.md. Never populated against a real deployment.
    public let devCode: String?
}

struct OtpVerifyBody: Encodable {
    let tenantSlug: String
    let phone: String
    let code: String
}

struct LoginBody: Encodable {
    let email: String
    let password: String
    let tenantSlug: String
}

struct RefreshBody: Encodable {
    let refreshToken: String
}

struct LinkPhoneBody: Encodable {
    let phone: String
}

struct SetAvailabilityBody: Encodable {
    let online: Bool
}

public struct UpdateLocationBody: Encodable, Sendable {
    public let latitude: Double
    public let longitude: Double
    public let speedKmh: Double?
    public let batteryPct: Double?

    public init(latitude: Double, longitude: Double, speedKmh: Double?, batteryPct: Double?) {
        self.latitude = latitude
        self.longitude = longitude
        self.speedKmh = speedKmh
        self.batteryPct = batteryPct
    }
}

public struct DeliverOrderBody: Encodable, Sendable {
    public let recipientName: String
    public let qrBarcodeVerified: Bool
    public let digitalSignatureBase64: String?
    public let photoEvidenceUrl: String?
    public let paymentCollected: Bool

    public init(
        recipientName: String,
        qrBarcodeVerified: Bool,
        digitalSignatureBase64: String?,
        photoEvidenceUrl: String?,
        paymentCollected: Bool
    ) {
        self.recipientName = recipientName
        self.qrBarcodeVerified = qrBarcodeVerified
        self.digitalSignatureBase64 = digitalSignatureBase64
        self.photoEvidenceUrl = photoEvidenceUrl
        self.paymentCollected = paymentCollected
    }
}

public struct CompletePickupBody: Encodable, Sendable {
    public let pickupPhotoEvidenceUrl: String

    public init(pickupPhotoEvidenceUrl: String) {
        self.pickupPhotoEvidenceUrl = pickupPhotoEvidenceUrl
    }
}

struct RegisterPushDeviceBody: Encodable {
    let platform: String
    /// "courier" or "customer" — see `AppVariant`. Tells the backend which
    /// bundle id's `apns-topic` this token belongs to.
    let app: String
    let deviceToken: String
}

struct UploadedFileResponse: Decodable {
    let url: String
}

/// Empty JSON object body (`{}`), used for endpoints that ignore any body.
struct EmptyBody: Encodable {}

// MARK: - Customer

struct RegisterAccountBody: Encodable {
    let email: String
    let displayName: String
    let password: String
    let tenantSlug: String?
}

struct AddAddressBody: Encodable {
    let label: String
    let latitude: Double
    let longitude: Double
    let fullAddress: String
}

public struct CreateCustomerOrderBody: Encodable, Sendable {
    public let pickup: Address
    public let dropoff: Address
    public let couponCode: String?
    public let paymentMethod: String?
    public let deliveryNote: String?
    public let contactPhone: String?

    public init(
        pickup: Address,
        dropoff: Address,
        couponCode: String?,
        paymentMethod: PaymentMethod?,
        deliveryNote: String?,
        contactPhone: String?
    ) {
        self.pickup = pickup
        self.dropoff = dropoff
        self.couponCode = couponCode
        self.paymentMethod = paymentMethod?.rawValue
        self.deliveryNote = deliveryNote
        self.contactPhone = contactPhone
    }
}

struct RateOrderBody: Encodable {
    let ratingStars: Int
    let comment: String?
}

struct OpenSupportTicketBody: Encodable {
    let orderId: UUID?
    let subject: String
    let message: String
}
