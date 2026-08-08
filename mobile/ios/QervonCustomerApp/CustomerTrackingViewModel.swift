// =============================================================================
// File:           mobile/ios/QervonCustomerApp/CustomerTrackingViewModel.swift
// Project:        Qervon Logistics Operating System (LOS)
// Author:         USDTG GROUP TECHNOLOGY LLC / Irfan Gedik
// Description:    Native iOS Customer Tracking ViewModel with Protected WS Stream
// =============================================================================

import Foundation
import CoreLocation
import MapKit
import Combine

class CustomerTrackingViewModel: ObservableObject {
    @Published var courierCoordinate: CLLocationCoordinate2D?
    @Published var courierName: String = "Ahmet Kurye (Motor 🏍️)"
    @Published var etaMinutes: Int = 3
    @Published var statusText: String = "Sunucu Korumalı GPS Takip Ediliyor 🛡️"
    
    let assignedCourierId = "00000000-0000-0000-0000-000000000001"
    let wsBaseUrl = "ws://localhost:8080/ws/tracking/customer"
    
    private var webSocketTask: URLSessionWebSocketTask?

    init() {
        connectToProtectedWebSocket()
    }

    private func connectToProtectedWebSocket() {
        guard let url = URL(string: "\(wsBaseUrl)?courier_id=\(assignedCourierId)") else { return }
        let session = URLSession(configuration: .default)
        webSocketTask = session.webSocketTask(with: url)
        webSocketTask?.resume()
        
        receiveLocationMessage()
    }

    private func receiveLocationMessage() {
        webSocketTask?.receive { [weak self] result in
            switch result {
            case .success(let message):
                switch message {
                case .string(let text):
                    self?.handleIncomingJSON(text)
                case .data(let data):
                    if let text = String(data: data, encoding: .utf8) {
                        self?.handleIncomingJSON(text)
                    }
                @unknown default:
                    break
                }
                // Continue listening loop
                self?.receiveLocationMessage()
            case .failure(let error):
                print("iOS Customer WS Receiver Error: \(error)")
            }
        }
    }

    private func handleIncomingJSON(_ jsonString: String) {
        guard let data = jsonString.data(using: .utf8),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let lat = json["latitude"] as? Double,
              let lon = json["longitude"] as? Double else { return }

        DispatchQueue.main.async {
            self.courierCoordinate = CLLocationCoordinate2D(latitude: lat, longitude: lon)
            self.statusText = String(format: "Atanan Kurye Konumu: Lat %.5f, Lon %.5f", lat, lon)
        }
    }
}
