// swift-tools-version:5.9
// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Package.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Cross-cutting infrastructure for Qervon's native iOS apps: shared DTOs,
//   the backend HTTP client, Keychain-backed session storage, the courier
//   GPS broadcaster, and shared UI theming. Consumed by Features/QervonFeatures
//   and the app targets. This package intentionally has no UI-flow logic of
//   its own — that belongs in Features.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import PackageDescription

let package = Package(
    name: "QervonKit",
    platforms: [.iOS(.v16), .macOS(.v13)],
    products: [
        .library(name: "QervonCore", targets: ["QervonCore"]),
        .library(name: "QervonNetworking", targets: ["QervonNetworking"]),
        .library(name: "QervonSecurity", targets: ["QervonSecurity"]),
        .library(name: "QervonLocation", targets: ["QervonLocation"]),
        .library(name: "QervonDesignSystem", targets: ["QervonDesignSystem"]),
    ],
    targets: [
        .target(name: "QervonCore"),
        .target(name: "QervonNetworking", dependencies: ["QervonCore"]),
        .target(name: "QervonSecurity", dependencies: ["QervonCore"]),
        .target(name: "QervonLocation", dependencies: ["QervonCore", "QervonNetworking"]),
        .target(name: "QervonDesignSystem", dependencies: ["QervonCore"]),
        .testTarget(name: "QervonCoreTests", dependencies: ["QervonCore"]),
        .testTarget(name: "QervonNetworkingTests", dependencies: ["QervonNetworking"]),
    ]
)
