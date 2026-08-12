// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/MapsFeature/NavigationLauncher.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Deep-links into Apple Maps, Google Maps, or Yandex Navi for
//   turn-by-turn navigation to a pickup/dropoff point. There is no backend
//   involvement — this is purely a URL scheme handoff to whichever app is
//   installed; Apple Maps is always available as the guaranteed fallback.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import UIKit
import QervonCore

public enum NavigationApp: String, CaseIterable, Identifiable {
    case appleMaps = "Apple Haritalar"
    case googleMaps = "Google Maps"
    case yandexNavi = "Yandex Navigasyon"

    public var id: String { rawValue }

    func url(to destination: GeoLocation) -> URL? {
        let lat = destination.latitude
        let lon = destination.longitude
        switch self {
        case .appleMaps:
            return URL(string: "maps://?daddr=\(lat),\(lon)&dirflg=d")
        case .googleMaps:
            return URL(string: "comgooglemaps://?daddr=\(lat),\(lon)&directionsmode=driving")
        case .yandexNavi:
            return URL(string: "yandexnavi://build_route_on_map?lat_to=\(lat)&lon_to=\(lon)")
        }
    }

    /// Whether the corresponding app is installed and can handle this
    /// scheme. Apple Maps is always considered available (it ships with
    /// iOS), the others require `LSApplicationQueriesSchemes` entries in
    /// Info.plist (see Project.yml) to be queryable at all.
    @MainActor
    public func isAvailable(for destination: GeoLocation) -> Bool {
        guard let url = url(to: destination) else { return false }
        if self == .appleMaps { return true }
        return UIApplication.shared.canOpenURL(url)
    }
}

public enum NavigationLauncher {
    @MainActor
    public static func open(_ app: NavigationApp, to destination: GeoLocation) {
        guard let url = app.url(to: destination) else { return }
        UIApplication.shared.open(url)
    }
}
