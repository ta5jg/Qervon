// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Tests/QervonCoreTests/CustomerModelsTests.swift
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

final class CustomerModelsTests: XCTestCase {
    func testDecodesCustomerProfileShapeFromBackend() throws {
        let json = """
        {
            "id": "019ff5cd-f08b-73c2-8f77-07c852fbdaa1",
            "user_id": "019ff5cd-f08b-73c2-8f77-07c852fbdaa2",
            "company_name": null,
            "tax_id": null,
            "addresses": [
                {
                    "id": "019ff5cd-f08b-73c2-8f77-07c852fbdaa3",
                    "label": "Ev",
                    "location": { "latitude": 41.0, "longitude": 29.0 },
                    "full_address": "Sultanahmet, Fatih/İstanbul",
                    "is_default": true
                }
            ],
            "loyalty_points": 0,
            "created_at": "2026-08-12T10:00:00Z"
        }
        """.data(using: .utf8)!

        let profile = try QervonJSON.makeDecoder().decode(CustomerProfile.self, from: json)
        XCTAssertEqual(profile.addresses.count, 1)
        XCTAssertTrue(profile.addresses[0].isDefault)
        XCTAssertEqual(profile.addresses[0].label, "Ev")
    }

    func testDecodesFareQuoteAndEtaShapes() throws {
        let quoteJSON = """
        { "fare_amount_minor": 3500, "currency": "TRY", "distance_km": 10.0 }
        """.data(using: .utf8)!
        let quote = try QervonJSON.makeDecoder().decode(FareQuote.self, from: quoteJSON)
        XCTAssertEqual(quote.fareAmountMinor, 3500)
        XCTAssertEqual(quote.money.formatted.isEmpty, false)

        let etaJSON = """
        { "eta_minutes": 12.5, "distance_km": 5.2 }
        """.data(using: .utf8)!
        let eta = try QervonJSON.makeDecoder().decode(EtaInfo.self, from: etaJSON)
        XCTAssertEqual(eta.etaMinutes, 12.5)
    }

    func testDecodesSupportTicketAndNotificationShapes() throws {
        let ticketJSON = """
        {
            "id": "019ff5cd-f08b-73c2-8f77-07c852fbdaa1",
            "customer_id": "019ff5cd-f08b-73c2-8f77-07c852fbdaa2",
            "order_id": null,
            "subject": "Genel soru",
            "message": "Ödeme yöntemleri nelerdir?",
            "status": "open",
            "created_at": "2026-08-12T10:00:00Z"
        }
        """.data(using: .utf8)!
        let ticket = try QervonJSON.makeDecoder().decode(SupportTicket.self, from: ticketJSON)
        XCTAssertEqual(ticket.status, .open)
        XCTAssertNil(ticket.orderId)

        let notificationJSON = """
        {
            "id": "019ff5cd-f08b-73c2-8f77-07c852fbdaa1",
            "recipient_id": "019ff5cd-f08b-73c2-8f77-07c852fbdaa2",
            "channel": "push",
            "title": "Siparişiniz yolda",
            "body": "Kuryeniz teslim noktasına yaklaşıyor.",
            "status": "sent",
            "created_at": "2026-08-12T10:00:00Z",
            "sent_at": "2026-08-12T10:00:05Z"
        }
        """.data(using: .utf8)!
        let notification = try QervonJSON.makeDecoder().decode(AppNotification.self, from: notificationJSON)
        XCTAssertEqual(notification.channel, .push)
        XCTAssertEqual(notification.status, .sent)
    }
}
