// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/AddressBookFeature/MapAddressPickerView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   A real MapKit-based address picker: search by name (MKLocalSearch),
//   pan/zoom to refine, then reverse-geocode the pinned center (CLGeocoder)
//   into a human-readable address. No backend involvement — this is pure
//   on-device Apple Maps/geocoding.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import MapKit
import QervonCore
import QervonDesignSystem

public struct MapAddressPickerView: View {
    let onPicked: (GeoLocation, String) -> Void
    @Environment(\.dismiss) private var dismiss

    @State private var region = MKCoordinateRegion(
        center: CLLocationCoordinate2D(latitude: 41.0082, longitude: 28.9784),
        span: MKCoordinateSpan(latitudeDelta: 0.05, longitudeDelta: 0.05)
    )
    @State private var searchQuery = ""
    @State private var isSearching = false
    @State private var isResolving = false
    @State private var errorMessage: String?

    public init(onPicked: @escaping (GeoLocation, String) -> Void) {
        self.onPicked = onPicked
    }

    public var body: some View {
        ZStack {
            Map(coordinateRegion: $region)
                .ignoresSafeArea()

            Image(systemName: "mappin.circle.fill")
                .font(.system(size: 32))
                .foregroundColor(QervonColor.danger)
                .offset(y: -16)
                .allowsHitTesting(false)

            VStack {
                HStack {
                    TextField("Adres veya yer ara", text: $searchQuery)
                        .textFieldStyle(.plain)
                        .padding(10)
                        .background(.white)
                        .cornerRadius(10)
                        .onSubmit { Task { await search() } }
                    Button {
                        Task { await search() }
                    } label: {
                        Image(systemName: "magnifyingglass")
                            .foregroundColor(.white)
                            .padding(10)
                            .background(QervonColor.accent)
                            .cornerRadius(10)
                    }
                }
                .padding()

                Spacer()

                VStack(spacing: QervonSpacing.sm) {
                    if let errorMessage {
                        Text(errorMessage)
                            .font(.system(size: 12))
                            .foregroundColor(.white)
                            .padding(.horizontal, QervonSpacing.md)
                    }
                    Button(isResolving ? "Konum çözülüyor…" : "Bu Konumu Seç") {
                        Task { await confirmSelection() }
                    }
                    .buttonStyle(QervonButtonStyle(isEnabled: !isResolving))
                    .disabled(isResolving)
                    .padding(.horizontal, QervonSpacing.lg)
                    .padding(.bottom, QervonSpacing.lg)
                }
                .background(QervonColor.background.opacity(0.85))
            }
        }
    }

    private func search() async {
        guard !searchQuery.trimmingCharacters(in: .whitespaces).isEmpty else { return }
        isSearching = true
        errorMessage = nil
        defer { isSearching = false }
        let request = MKLocalSearch.Request()
        request.naturalLanguageQuery = searchQuery
        request.region = region
        do {
            let response = try await MKLocalSearch(request: request).start()
            guard let coordinate = response.mapItems.first?.placemark.coordinate else {
                errorMessage = "Sonuç bulunamadı."
                return
            }
            withAnimation {
                region = MKCoordinateRegion(
                    center: coordinate,
                    span: MKCoordinateSpan(latitudeDelta: 0.01, longitudeDelta: 0.01)
                )
            }
        } catch {
            errorMessage = error.localizedDescription
        }
    }

    private func confirmSelection() async {
        isResolving = true
        errorMessage = nil
        defer { isResolving = false }
        let center = region.center
        let location = CLLocation(latitude: center.latitude, longitude: center.longitude)
        let geocoder = CLGeocoder()
        do {
            let placemarks = try await geocoder.reverseGeocodeLocation(location)
            let fullAddress = placemarks.first?.qervonFormattedAddress
                ?? String(format: "%.5f, %.5f", center.latitude, center.longitude)
            onPicked(GeoLocation(latitude: center.latitude, longitude: center.longitude), fullAddress)
            dismiss()
        } catch {
            // Reverse geocoding can fail offline; fall back to raw
            // coordinates rather than blocking address selection entirely.
            let fullAddress = String(format: "%.5f, %.5f", center.latitude, center.longitude)
            onPicked(GeoLocation(latitude: center.latitude, longitude: center.longitude), fullAddress)
            dismiss()
        }
    }
}

private extension CLPlacemark {
    var qervonFormattedAddress: String {
        [thoroughfare, subLocality, locality]
            .compactMap { $0 }
            .joined(separator: ", ")
    }
}
