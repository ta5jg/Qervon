// =============================================================================
// File:           mobile/ios/QervonCourierApp/CourierView.swift
// Project:        Qervon Logistics Operating System (LOS)
// Author:         USDTG GROUP TECHNOLOGY LLC / Irfan Gedik
// Description:    Native SwiftUI Courier Terminal Interface
// =============================================================================

import SwiftUI
import MapKit

struct CourierView: View {
    @StateObject private var broadcaster = CourierLocationBroadcaster()
    @State private var region = MKCoordinateRegion(
        center: CLLocationCoordinate2D(latitude: 41.0638, longitude: 28.9351), // Default Yıldıztabya
        span: MKCoordinateSpan(latitudeDelta: 0.01, longitudeDelta: 0.01)
    )

    var body: some View {
        ZStack {
            Color(red: 0.02, green: 0.03, blue: 0.07).edgesIgnoringSafeArea(.all)

            VStack(spacing: 16) {
                // Header Bar
                HStack {
                    VStack(alignment: .leading, spacing: 2) {
                        Text("QERVON KURYE")
                            .font(.system(size: 18, weight: .bold))
                            .foregroundColor(.white)
                        Text("Terminal v1.0 • iOS Native")
                            .font(.system(size: 11, weight: .semibold))
                            .foregroundColor(Color(red: 0.22, green: 0.74, blue: 0.97))
                    }
                    Spacer()
                    Circle()
                        .fill(broadcaster.isBroadcasting ? Color.green : Color.red)
                        .frame(width: 12, height: 12)
                        .shadow(color: broadcaster.isBroadcasting ? .green : .red, radius: 8)
                }
                .padding(.horizontal, 20)
                .padding(.top, 10)

                // GPS Status Card
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        Text("DONANIM FİZİKSEL GPS YAYINI")
                            .font(.system(size: 10, weight: .bold))
                            .foregroundColor(.gray)
                        Spacer()
                        Text("HARDWARE LIVE")
                            .font(.system(size: 9, weight: .bold))
                            .padding(.horizontal, 8)
                            .padding(.vertical, 3)
                            .background(Color.green.opacity(0.2))
                            .foregroundColor(.green)
                            .cornerRadius(8)
                    }

                    Text(broadcaster.gpsStatusText)
                        .font(.system(size: 14, weight: .bold, design: .monospaced))
                        .foregroundColor(.green)
                }
                .padding(16)
                .background(Color(red: 0.06, green: 0.09, blue: 0.16).opacity(0.8))
                .cornerRadius(16)
                .overlay(RoundedRectangle(cornerRadius: 16).stroke(Color.white.opacity(0.1), lineWidth: 1))
                .padding(.horizontal, 20)

                // AI Dispatch Assignment Banner
                VStack(alignment: .leading, spacing: 6) {
                    Text("YENİ GÖREV BİLDİRİMİ (AI DISPATCH)")
                        .font(.system(size: 10, weight: .bold))
                        .foregroundColor(.gray)
                    Text("Sultanahmet Restoran ➔ Maslak Plaza")
                        .font(.system(size: 14, weight: .bold))
                        .foregroundColor(.white)
                    HStack {
                        Text("Kazanç: ₺45.00")
                            .font(.system(size: 12, weight: .bold))
                            .foregroundColor(Color(red: 0.22, green: 0.74, blue: 0.97))
                        Spacer()
                        Text("Tahmini: 11 Dk")
                            .font(.system(size: 12, weight: .semibold))
                            .foregroundColor(.gray)
                    }
                }
                .padding(16)
                .background(Color(red: 0.06, green: 0.09, blue: 0.16).opacity(0.8))
                .cornerRadius(16)
                .overlay(RoundedRectangle(cornerRadius: 16).stroke(Color(red: 0.22, green: 0.74, blue: 0.97).opacity(0.3), lineWidth: 1))
                .padding(.horizontal, 20)

                // Map View
                Map(coordinateRegion: $region, showsUserLocation: true)
                    .cornerRadius(20)
                    .overlay(RoundedRectangle(cornerRadius: 20).stroke(Color.white.opacity(0.1), lineWidth: 1))
                    .padding(.horizontal, 20)

                // Action Buttons
                Button(action: {
                    if broadcaster.isBroadcasting {
                        broadcaster.stopBroadcasting()
                    } else {
                        broadcaster.startBroadcasting()
                    }
                }) {
                    Text(broadcaster.isBroadcasting ? "GÖREVİ DURDUR / OFFLINE" : "GÖREVE BAŞLA & CANLI GPS YAYINLA")
                        .font(.system(size: 14, weight: .bold))
                        .foregroundColor(.white)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 16)
                        .background(
                            LinearGradient(
                                gradient: Gradient(colors: broadcaster.isBroadcasting ? [Color.red, Color.orange] : [Color(red: 0.06, green: 0.72, blue: 0.51), Color(red: 0.02, green: 0.58, blue: 0.41)]),
                                startPoint: .leading,
                                endPoint: .trailing
                            )
                        )
                        .cornerRadius(16)
                        .shadow(color: broadcaster.isBroadcasting ? .red.opacity(0.4) : .green.opacity(0.4), radius: 10)
                }
                .padding(.horizontal, 20)
                .padding(.bottom, 20)
            }
        }
        .onAppear {
            broadcaster.startBroadcasting()
        }
    }
}
