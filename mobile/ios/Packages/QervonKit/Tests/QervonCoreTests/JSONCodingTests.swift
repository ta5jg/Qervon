// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Tests/QervonCoreTests/JSONCodingTests.swift
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
@testable import QervonCore

final class JSONCodingTests: XCTestCase {
    func testDecodesOrderResponseShapeFromBackend() throws {
        let json = """
        {
            "id": "019ff5cd-f08b-73c2-8f77-07c852fbdaa1",
            "customer_id": "019ff5cd-f08b-73c2-8f77-07c852fbdaa2",
            "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
            "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
            "status": "courier_assigned",
            "fare": { "amount_minor": 4500, "currency": "TRY" },
            "assigned_courier_id": "019ff5cd-f08b-73c2-8f77-07c852fbdaa3",
            "created_at": "2026-08-12T10:00:00Z",
            "delivered_at": null,
            "returned_at": null,
            "payment_method": "cash",
            "payment_collected": false
        }
        """.data(using: .utf8)!

        let order = try QervonJSON.makeDecoder().decode(Order.self, from: json)
        XCTAssertEqual(order.status, .courierAssigned)
        XCTAssertEqual(order.fare.amountMinor, 4500)
        XCTAssertEqual(order.paymentMethod, .cash)
        XCTAssertNil(order.deliveredAt)
    }

    func testDecodesFractionalSecondsTimestamp() throws {
        let json = """
        { "value": "2026-08-12T10:00:00.123456Z" }
        """.data(using: .utf8)!
        struct Wrapper: Decodable { let value: Date }
        let wrapper = try QervonJSON.makeDecoder().decode(Wrapper.self, from: json)
        XCTAssertGreaterThan(wrapper.value.timeIntervalSince1970, 0)
    }

    func testAccessTokenClaimsDecodeWithoutSecret() throws {
        // A hand-built qv1 token whose payload matches the backend's
        // AccessClaims shape; signature verification is intentionally not
        // performed client-side (see Models/Auth.swift).
        let payload = """
        {"subject":"019ff5cd-f08b-73c2-8f77-07c852fbdaa1","tenant_id":"019ff5cd-f08b-73c2-8f77-07c852fbdaa2","role":"courier","expires_at":9999999999}
        """
        let encoded = Data(payload.utf8).base64EncodedString()
            .replacingOccurrences(of: "+", with: "-")
            .replacingOccurrences(of: "/", with: "_")
            .replacingOccurrences(of: "=", with: "")
        let token = "qv1.\(encoded).fakesignature"
        let claims = try AccessTokenClaims.decode(fromAccessToken: token)
        XCTAssertEqual(claims.role, .courier)
        XCTAssertFalse(claims.isExpired)
    }
}
