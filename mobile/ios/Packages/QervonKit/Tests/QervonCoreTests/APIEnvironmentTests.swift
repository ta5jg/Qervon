// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Tests/QervonCoreTests/APIEnvironmentTests.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-23
// Version:        0.1.0
//
// Description:
//   Guards the physical-device networking contract: Xcode Debug builds must
//   connect to the live API unless a developer explicitly saves an override.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import XCTest
@testable import QervonCore

final class APIEnvironmentTests: XCTestCase {
    override func tearDown() {
        APIEnvironment.setOverride(nil)
        super.tearDown()
    }

    func testDefaultBaseURLUsesLiveHTTPSAPI() {
        APIEnvironment.setOverride(nil)

        XCTAssertEqual(APIEnvironment.defaultBaseURL.absoluteString, "https://qervon.io")
        XCTAssertEqual(APIEnvironment.baseURL.absoluteString, "https://qervon.io")
    }

    func testExplicitDevelopmentOverrideIsTrimmedAndApplied() {
        APIEnvironment.setOverride("  http://192.168.1.20:8080\n")

        XCTAssertEqual(APIEnvironment.currentOverride(), "http://192.168.1.20:8080")
        XCTAssertEqual(APIEnvironment.baseURL.absoluteString, "http://192.168.1.20:8080")
    }

    func testInvalidOverrideDoesNotReplaceCurrentEnvironment() {
        APIEnvironment.setOverride(nil)
        APIEnvironment.setOverride("not a URL")

        XCTAssertNil(APIEnvironment.currentOverride())
        XCTAssertEqual(APIEnvironment.baseURL, APIEnvironment.defaultBaseURL)
    }
}
