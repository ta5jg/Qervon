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
//   Two reliability measures beyond a plain fire-and-forget POST:
//   - Each send is wrapped in a `UIApplication.beginBackgroundTask` so the
//     OS grants a grace period to finish the network request even if a
//     location callback fires right as the app is about to be suspended
//     (e.g. the courier switches to another app) — without this, an
//     in-flight request can be killed mid-flight.
//   - Failed sends are held in a small bounded retry queue (oldest-first,
//     capped at `maxPendingSamples`) instead of being dropped, so a brief
//     connectivity gap does not silently lose location beats; they are
//     flushed on the next tick. Not persisted to disk — this smooths over
//     seconds-scale gaps, not an app kill or device reboot.
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

private struct PendingLocationSample {
    let latitude: Double
    let longitude: Double
    let speedKmh: Double?
    let batteryPct: Double?
}

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

    private var pendingSamples: [PendingLocationSample] = []
    private let maxPendingSamples = 20
    private var isFlushingSamples = false

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
        let sample = PendingLocationSample(
            latitude: location.coordinate.latitude,
            longitude: location.coordinate.longitude,
            speedKmh: speedKmh,
            batteryPct: UIDeviceBattery.currentPercentage()
        )
        enqueueAndFlush(sample, tickTime: now)
    }

    private func enqueueAndFlush(_ sample: PendingLocationSample, tickTime: Date) {
        if pendingSamples.count >= maxPendingSamples {
            pendingSamples.removeFirst()
        }
        pendingSamples.append(sample)

        #if os(iOS)
        var backgroundTaskId: UIBackgroundTaskIdentifier = .invalid
        backgroundTaskId = UIApplication.shared.beginBackgroundTask(withName: "qervon.courier.location-report") {
            if backgroundTaskId != .invalid {
                UIApplication.shared.endBackgroundTask(backgroundTaskId)
                backgroundTaskId = .invalid
            }
        }
        #endif

        Task {
            await flushPendingSamples(tickTime: tickTime)
            #if os(iOS)
            if backgroundTaskId != .invalid {
                UIApplication.shared.endBackgroundTask(backgroundTaskId)
                backgroundTaskId = .invalid
            }
            #endif
        }
    }

    /// Sends every queued sample oldest-first, stopping at the first
    /// failure so ordering is preserved and a single tick does not hammer
    /// a still-unreachable backend repeatedly; the remaining queue is
    /// retried on the next location callback.
    private func flushPendingSamples(tickTime: Date) async {
        guard !isFlushingSamples else { return }
        isFlushingSamples = true
        defer { isFlushingSamples = false }

        while let sample = pendingSamples.first {
            let body = UpdateLocationBody(
                latitude: sample.latitude,
                longitude: sample.longitude,
                speedKmh: sample.speedKmh,
                batteryPct: sample.batteryPct
            )
            do {
                _ = try await api.updateOwnLocation(body)
                pendingSamples.removeFirst()
                statusText = "Konum yayınlanıyor · \(QervonFormat.time(tickTime))"
            } catch {
                statusText = "Konum g\u{00f6}nderilemedi, yeniden denenecek: \(error.localizedDescription)"
                break
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
