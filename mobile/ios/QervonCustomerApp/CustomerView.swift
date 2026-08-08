// =============================================================================
// File:           mobile/ios/QervonCustomerApp/CustomerView.swift
// Project:        Qervon Logistics Operating System (LOS)
// Author:         USDTG GROUP TECHNOLOGY LLC / Irfan Gedik
// Description:    Native SwiftUI Customer Application Main Interface
// =============================================================================

import SwiftUI
import MapKit

struct CustomerView: View {
    @StateObject private var viewModel = CustomerTrackingViewModel()
    @State private var selectedPackage: String = "Evrak"
    @State private var fare: Int = 45

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
                        Text("QERVON MÜŞTERİ")
                            .font(.system(size: 18, weight: .bold))
                            .foregroundColor(Color(red: 0.06, green: 0.72, blue: 0.51))
                        Text("iOS Native App • Sunucu Korumalı GPS")
                            .font(.system(size: 11, weight: .semibold))
                            .foregroundColor(.gray)
                    }
                    Spacer()
                    Text("LIVE HARDWARE 🎯")
                        .font(.system(size: 9, weight: .bold))
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(Color.green.opacity(0.15))
                        .foregroundColor(.green)
                        .cornerRadius(8)
                }
                .padding(.horizontal, 20)
                .padding(.top, 10)

                // Map View
                Map(coordinateRegion: $region)
                    .cornerRadius(20)
                    .overlay(RoundedRectangle(cornerRadius: 20).stroke(Color.white.opacity(0.1), lineWidth: 1))
                    .padding(.horizontal, 20)

                // Assigned Courier Live Card
                VStack(alignment: .leading, spacing: 8) {
                    HStack {
                        VStack(alignment: .leading, spacing: 2) {
                            Text("SADECE ATANAN KURYE (SUNUCU KORUMALI)")
                                .font(.system(size: 10, weight: .bold))
                                .foregroundColor(.gray)
                            Text(viewModel.courierName)
                                .font(.system(size: 14, weight: .bold))
                                .foregroundColor(Color(red: 0.06, green: 0.72, blue: 0.51))
                        }
                        Spacer()
                        VStack(alignment: .trailing, spacing: 2) {
                            Text("TAHMİNİ ETA")
                                .font(.system(size: 10, weight: .bold))
                                .foregroundColor(.gray)
                            Text("\(viewModel.etaMinutes) Dk")
                                .font(.system(size: 16, weight: .bold))
                                .foregroundColor(Color(red: 0.22, green: 0.74, blue: 0.97))
                        }
                    }

                    Text(viewModel.statusText)
                        .font(.system(size: 11, weight: .semibold, design: .monospaced))
                        .foregroundColor(.gray)
                }
                .padding(16)
                .background(Color(red: 0.06, green: 0.09, blue: 0.16).opacity(0.8))
                .cornerRadius(16)
                .overlay(RoundedRectangle(cornerRadius: 16).stroke(Color(red: 0.06, green: 0.72, blue: 0.51).opacity(0.3), lineWidth: 1))
                .padding(.horizontal, 20)

                // Package Selection Cards
                VStack(alignment: .leading, spacing: 8) {
                    Text("Paket Tipi Seçimi")
                        .font(.system(size: 12, weight: .bold))
                        .foregroundColor(.gray)

                    HStack(spacing: 10) {
                        PackageOptionCard(title: "Evrak", icon: "doc.text", price: 45, isSelected: selectedPackage == "Evrak") {
                            selectedPackage = "Evrak"; fare = 45
                        }
                        PackageOptionCard(title: "Gıda", icon: "fork.knife", price: 65, isSelected: selectedPackage == "Gıda") {
                            selectedPackage = "Gıda"; fare = 65
                        }
                        PackageOptionCard(title: "Koli", icon: "box.truck", price: 120, isSelected: selectedPackage == "Koli") {
                            selectedPackage = "Koli"; fare = 120
                        }
                    }
                }
                .padding(.horizontal, 20)

                // Call Courier Button
                Button(action: {
                    // Call courier action
                }) {
                    Text("⚡ KURYEYİ ÇAĞIR (₺\(fare).00)")
                        .font(.system(size: 14, weight: .bold))
                        .foregroundColor(.white)
                        .frame(maxWidth: .infinity)
                        .padding(.vertical, 16)
                        .background(
                            LinearGradient(
                                gradient: Gradient(colors: [Color(red: 0.06, green: 0.72, blue: 0.51), Color(red: 0.02, green: 0.58, blue: 0.41)]),
                                startPoint: .leading,
                                endPoint: .trailing
                            )
                        )
                        .cornerRadius(16)
                        .shadow(color: Color.green.opacity(0.4), radius: 10)
                }
                .padding(.horizontal, 20)
                .padding(.bottom, 20)
            }
        }
    }
}

struct PackageOptionCard: View {
    let title: String
    let icon: String
    let price: Int
    let isSelected: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            VStack(spacing: 4) {
                Image(systemName: icon)
                    .font(.system(size: 16))
                    .foregroundColor(isSelected ? .green : .gray)
                Text(title)
                    .font(.system(size: 11, weight: .bold))
                    .foregroundColor(.white)
                Text("₺\(price)")
                    .font(.system(size: 10, weight: .semibold))
                    .foregroundColor(.gray)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 10)
            .background(isSelected ? Color.green.opacity(0.15) : Color(red: 0.06, green: 0.09, blue: 0.16))
            .cornerRadius(12)
            .overlay(RoundedRectangle(cornerRadius: 12).stroke(isSelected ? Color.green : Color.white.opacity(0.1), lineWidth: 1.5))
        }
    }
}
