// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Tests/QervonNetworkingTests/RequestEncodingTests.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import XCTest
import QervonCore
@testable import QervonNetworking

final class RequestEncodingTests: XCTestCase {
    func testOtpRequestBodyEncodesSnakeCase() throws {
        let body = OtpRequestBody(tenantSlug: "acme", phone: "+905551234567")
        let data = try QervonJSON.makeEncoder().encode(body)
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        XCTAssertEqual(json?["tenant_slug"] as? String, "acme")
        XCTAssertEqual(json?["phone"] as? String, "+905551234567")
    }

    func testDeliverOrderBodyEncodesSnakeCase() throws {
        let body = DeliverOrderBody(
            recipientName: "Ali Veli",
            qrBarcodeVerified: true,
            digitalSignatureBase64: nil,
            photoEvidenceUrl: nil,
            paymentCollected: true
        )
        let data = try QervonJSON.makeEncoder().encode(body)
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        XCTAssertEqual(json?["recipient_name"] as? String, "Ali Veli")
        XCTAssertEqual(json?["qr_barcode_verified"] as? Bool, true)
        XCTAssertEqual(json?["payment_collected"] as? Bool, true)
    }

    func testCompletePickupBodyEncodesEvidenceURL() throws {
        let body = CompletePickupBody(pickupPhotoEvidenceUrl: "/v1/uploads/delivery-photos/order/photo.jpg")
        let data = try QervonJSON.makeEncoder().encode(body)
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        XCTAssertEqual(
            json?["pickup_photo_evidence_url"] as? String,
            "/v1/uploads/delivery-photos/order/photo.jpg"
        )
    }

    func testOtpRequestResultDecodesDevCode() throws {
        let json = """
        { "status": "sent", "dev_code": "123456" }
        """.data(using: .utf8)!
        let result = try QervonJSON.makeDecoder().decode(OtpRequestResult.self, from: json)
        XCTAssertEqual(result.status, "sent")
        XCTAssertEqual(result.devCode, "123456")
    }

    func testCreateCustomerOrderBodyOmitsClientSuppliedFare() throws {
        let body = CreateCustomerOrderBody(
            pickup: Address(latitude: 41.0, longitude: 29.0, label: "Alım"),
            dropoff: Address(latitude: 41.1, longitude: 29.1, label: "Teslim"),
            couponCode: "QERVON20",
            paymentMethod: .cash,
            deliveryNote: "Kapıcıya bırakın",
            contactPhone: "+905551234567"
        )
        let data = try QervonJSON.makeEncoder().encode(body)
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        XCTAssertNil(json?["fare_amount_minor"])
        XCTAssertNil(json?["fare_currency"])
        XCTAssertEqual(json?["payment_method"] as? String, "cash")
        XCTAssertEqual(json?["delivery_note"] as? String, "Kapıcıya bırakın")
    }

    func testRegisterAccountBodyEncodesSnakeCaseWithOptionalTenantSlug() throws {
        let body = RegisterAccountBody(
            email: "test@example.com",
            displayName: "Test User",
            password: "supersecretpassword",
            tenantSlug: "acme"
        )
        let data = try QervonJSON.makeEncoder().encode(body)
        let json = try JSONSerialization.jsonObject(with: data) as? [String: Any]
        XCTAssertEqual(json?["display_name"] as? String, "Test User")
        XCTAssertEqual(json?["tenant_slug"] as? String, "acme")
    }
}
