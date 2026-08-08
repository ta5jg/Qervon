// =============================================================================
// File:           mobile/ios/QervonCourierApp/CourierLocationBroadcaster.swift
// Project:        Qervon Logistics Operating System (LOS)
// Author:         USDTG GROUP TECHNOLOGY LLC / Irfan Gedik
// Description:    Native iOS CoreLocation Hardware GPS Broadcaster Service
// =============================================================================

import Foundation
import CoreLocation
import Combine

class CourierLocationBroadcaster: NSObject, ObservableObject, CLLocationManagerDelegate {
    private let locationManager = CLLocationManager()
    private var webSocketTask: URLSessionWebSocketTask?
    
    @Published var currentLocation: CLLocationCoordinate2D?
    @Published var isBroadcasting: Bool = false
    @Published var gpsStatusText: String = "GPS Hazır"
    
    let courierId = "00000000-0000-0000-0000-000000000001"
    let apiBaseUrl = "http://localhost:8080"
    let wsBaseUrl = "ws://localhost:8080/ws/tracking"

    override init() {
        super.init()
        setupLocationManager()
        connectWebSocket()
    }

    private func setupLocationManager() {
        locationManager.delegate = self
        locationManager.desiredAccuracy = kCLLocationAccuracyBestForNavigation
        locationManager.distanceFilter = 1.0 // Broadcast every 1 meter movement
        locationManager.allowsBackgroundLocationUpdates = true
        locationManager.showsBackgroundLocationIndicator = true
        locationManager.requestAlwaysAuthorization()
    }

    func startBroadcasting() {
        locationManager.startUpdatingLocation()
        isBroadcasting = true
        gpsStatusText = "Donanım GPS Yayınlanıyor 🛰️"
    }

    func stopBroadcasting() {
        locationManager.stopUpdatingLocation()
        isBroadcasting = false
        gpsStatusText = "GPS Durduruldu"
    }

    private func connectWebSocket() {
        guard let url = URL(string: wsBaseUrl) else { return }
        let session = URLSession(configuration: .default)
        webSocketTask = session.webSocketTask(with: url)
        webSocketTask?.resume()
    }

    // CoreLocation Delegate Callback
    func locationManager(_ manager: CLLocationManager, didUpdateLocations locations: [CLLocation]) {
        guard let location = locations.last else { return }
        DispatchQueue.main.async {
            self.currentLocation = location.coordinate
            self.gpsStatusText = String(format: "Lat: %.5f, Lon: %.5f", location.coordinate.latitude, location.coordinate.longitude)
        }
        
        // Broadcast location to Rust Backend API & WebSocket Stream
        sendLocationToBackend(latitude: location.coordinate.latitude, longitude: location.coordinate.longitude)
    }

    private func sendLocationToBackend(latitude: Double, longitude: Double) {
        // 1. REST API Post
        guard let url = URL(string: "\(apiBaseUrl)/v1/couriers/\(courierId)/location") else { return }
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        
        let body: [String: Any] = ["latitude": latitude, "longitude": longitude]
        request.httpBody = try? JSONSerialization.data(withJSONObject: body)
        
        URLSession.shared.dataTask(with: request).resume()
        
        // 2. WebSocket Real-time Packet Broadcast
        let wsMessage: [String: Any] = [
            "courier_id": courierId,
            "latitude": latitude,
            "longitude": longitude
        ]
        if let jsonData = try? JSONSerialization.data(withJSONObject: wsMessage),
           let jsonString = String(data: jsonData, encoding: .utf8) {
            let message = URLSessionWebSocketTask.Message.string(jsonString)
            webSocketTask?.send(message) { error in
                if let error = error {
                    print("iOS WS Broadcast Error: \(error)")
                }
            }
        }
    }
}
