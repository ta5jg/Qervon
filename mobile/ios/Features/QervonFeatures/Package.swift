// swift-tools-version:5.9
// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Package.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   SwiftUI feature modules shared by the Qervon Courier app (Faz-2.2) and
//   the Qervon Customer app (Faz-2.3). Each target owns one screen area's
//   views and view models; all backend/session/location plumbing lives in
//   ../../Packages/QervonKit and is only consumed here. AuthFeature is
//   shared by both apps; the rest are app-specific.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import PackageDescription

let package = Package(
    name: "QervonFeatures",
    platforms: [.iOS(.v16)],
    products: [
        .library(name: "AuthFeature", targets: ["AuthFeature"]),
        .library(name: "DispatchFeature", targets: ["DispatchFeature"]),
        .library(name: "OrdersFeature", targets: ["OrdersFeature"]),
        .library(name: "MapsFeature", targets: ["MapsFeature"]),
        .library(name: "ProofOfDeliveryFeature", targets: ["ProofOfDeliveryFeature"]),
        .library(name: "EarningsFeature", targets: ["EarningsFeature"]),
        .library(name: "ProfileFeature", targets: ["ProfileFeature"]),
        .library(name: "AddressBookFeature", targets: ["AddressBookFeature"]),
        .library(name: "CustomerOrderFeature", targets: ["CustomerOrderFeature"]),
        .library(name: "CustomerProfileFeature", targets: ["CustomerProfileFeature"]),
    ],
    dependencies: [
        .package(path: "../../Packages/QervonKit"),
    ],
    targets: [
        .target(
            name: "AuthFeature",
            dependencies: [
                .product(name: "QervonCore", package: "QervonKit"),
                .product(name: "QervonNetworking", package: "QervonKit"),
                .product(name: "QervonSecurity", package: "QervonKit"),
                .product(name: "QervonDesignSystem", package: "QervonKit"),
            ]
        ),
        .target(
            name: "DispatchFeature",
            dependencies: [
                .product(name: "QervonCore", package: "QervonKit"),
                .product(name: "QervonNetworking", package: "QervonKit"),
                .product(name: "QervonLocation", package: "QervonKit"),
                .product(name: "QervonDesignSystem", package: "QervonKit"),
                "OrdersFeature",
            ]
        ),
        .target(
            name: "OrdersFeature",
            dependencies: [
                .product(name: "QervonCore", package: "QervonKit"),
                .product(name: "QervonNetworking", package: "QervonKit"),
                .product(name: "QervonDesignSystem", package: "QervonKit"),
                "MapsFeature",
                "ProofOfDeliveryFeature",
            ]
        ),
        .target(
            name: "MapsFeature",
            dependencies: [
                .product(name: "QervonCore", package: "QervonKit"),
                .product(name: "QervonDesignSystem", package: "QervonKit"),
            ]
        ),
        .target(
            name: "ProofOfDeliveryFeature",
            dependencies: [
                .product(name: "QervonCore", package: "QervonKit"),
                .product(name: "QervonNetworking", package: "QervonKit"),
                .product(name: "QervonDesignSystem", package: "QervonKit"),
            ]
        ),
        .target(
            name: "EarningsFeature",
            dependencies: [
                .product(name: "QervonCore", package: "QervonKit"),
                .product(name: "QervonNetworking", package: "QervonKit"),
                .product(name: "QervonDesignSystem", package: "QervonKit"),
            ]
        ),
        .target(
            name: "ProfileFeature",
            dependencies: [
                .product(name: "QervonCore", package: "QervonKit"),
                .product(name: "QervonNetworking", package: "QervonKit"),
                .product(name: "QervonSecurity", package: "QervonKit"),
                .product(name: "QervonDesignSystem", package: "QervonKit"),
            ]
        ),
        .target(
            name: "AddressBookFeature",
            dependencies: [
                .product(name: "QervonCore", package: "QervonKit"),
                .product(name: "QervonNetworking", package: "QervonKit"),
                .product(name: "QervonDesignSystem", package: "QervonKit"),
            ]
        ),
        .target(
            name: "CustomerOrderFeature",
            dependencies: [
                .product(name: "QervonCore", package: "QervonKit"),
                .product(name: "QervonNetworking", package: "QervonKit"),
                .product(name: "QervonDesignSystem", package: "QervonKit"),
                "AddressBookFeature",
            ]
        ),
        .target(
            name: "CustomerProfileFeature",
            dependencies: [
                .product(name: "QervonCore", package: "QervonKit"),
                .product(name: "QervonNetworking", package: "QervonKit"),
                .product(name: "QervonSecurity", package: "QervonKit"),
                .product(name: "QervonDesignSystem", package: "QervonKit"),
                "AddressBookFeature",
            ]
        ),
    ]
)
