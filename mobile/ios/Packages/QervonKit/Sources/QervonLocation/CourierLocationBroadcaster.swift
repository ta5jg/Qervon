// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonLocation/CourierLocationBroadcaster.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Hardware GPS broadcaster: reads real CoreLocation updates and forwards
//   them to `POST /v1/courier/me/location`. Runs while the courier is
//   online, including in the background (see the app target's
//   `UIBackgroundModes: [location]` entitlement).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import CoreLocation
#if canImport(UIKit)
import UIKit
#endif
import QervonCore
import QervonNetworking

@MainActor
public final class CourierLocationBroadcaster: NSObject, ObservableObject, @preconcurrency CLLocationManagerDelegate {
    @Published public private(set) var isBroadcasting = false
    @Published public private(set) var lastKnownLocation: CLLocation?
    @Published public private(set) var statusText = "GPS Hazır"
    @Published public private(set) var authorizationStatus: CLAuthorizationStatus

    private let locationManager = CLLocationManager()
    private let api: QervonAPI
    private var lastSentAt: Date?
    /// Avoids flooding the backend on every CoreLocation callback; a courier
    /// moving in a city does not need sub-second location resolution.
    private let minimumSendInterval: TimeInterval = 3

    public init(api: QervonAPI) {
        self.api = api
        self.authorizationStatus = locationManager.authorizationStatus
        super.init()
        locationManager.delegate = self
        locationManager.desiredAccuracy = kCLLocationAccuracyBestForNavigation
        locationManager.distanceFilter = 10
        locationManager.allowsBackgroundLocationUpdates = false
    }

    public func requestPermission() {
        locationManager.requestAlwaysAuthorization()
    }

    public func startBroadcasting() {
        guard !isBroadcasting else { return }
        #if os(iOS)
        locationManager.allowsBackgroundLocationUpdates = true
        locationManager.showsBackgroundLocationIndicator = true
        #endif
        locationManager.startUpdatingLocation()
        isBroadcasting = true
        statusText = "Konum yayınlanıyor"
    }

    public func stopBroadcasting() {
        guard isBroadcasting else { return }
        locationManager.stopUpdatingLocation()
        #if os(iOS)
        locationManager.allowsBackgroundLocationUpdates = false
        #endif
        isBroadcasting = false
        statusText = "Konum yayını durduruldu"
    }

    public func locationManagerDidChangeAuthorization(_ manager: CLLocationManager) {
        authorizationStatus = manager.authorizationStatus
    }

    public func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
        guard let location = locations.last else { return }
        lastKnownLocation = location

        let now = Date()
        if let lastSentAt, now.timeIntervalSince(lastSentAt) < minimumSendInterval {
            return
        }
        lastSentAt = now

        let speedKmh = location.speed >= 0 ? location.speed * 3.6 : nil
        let body = UpdateLocationBody(
            latitude: location.coordinate.latitude,
            longitude: location.coordinate.longitude,
            speedKmh: speedKmh,
            batteryPct: UIDeviceBattery.currentPercentage()
        )
        Task {
            do {
                _ = try await api.updateOwnLocation(body)
                statusText = "Konum yayınlanıyor · \(QervonFormat.time(now))"
            } catch {
                statusText = "Konum g\u{00f6}nderilemedi: \(error.localizedDescription)"
            }
        }
    }

    public func locationManager(_ manager: CLLocationManager, didFailWithError error: Error) {
        statusText = "GPS hatası: \(error.localizedDescription)"
    }
}

enum UIDeviceBattery {
    /// Battery percentage 0-100, or `nil` on Simulator/devices that report
    /// an unknown level (never fabricates a number).
    @MainActor
    static func currentPercentage() -> Double? {
        #if canImport(UIKit) && os(iOS)
        UIDevice.current.isBatteryMonitoringEnabled = true
        let level = UIDevice.current.batteryLevel
        guard level >= 0 else { return nil }
        return Double(level) * 100
        #else
        return nil
        #endif
    }
}
