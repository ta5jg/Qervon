// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/CustomerOrderFeature/OrderDetailView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import MapKit
import QervonCore
import QervonNetworking
import QervonDesignSystem

private struct OrderMapPin: Identifiable {
    enum Kind { case pickup, dropoff, courier }
    let id: String
    let kind: Kind
    let coordinate: CLLocationCoordinate2D
}

public struct OrderDetailView: View {
    @StateObject private var viewModel: OrderDetailViewModel
    @State private var region: MKCoordinateRegion
    @State private var showingRating = false
    @State private var showingSupport = false

    public init(order: Order, api: QervonAPI) {
        _viewModel = StateObject(wrappedValue: OrderDetailViewModel(order: order, api: api))
        _region = State(initialValue: MKCoordinateRegion(
            center: CLLocationCoordinate2D(latitude: order.pickup.latitude, longitude: order.pickup.longitude),
            span: MKCoordinateSpan(latitudeDelta: 0.05, longitudeDelta: 0.05)
        ))
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.lg) {
                Map(coordinateRegion: $region, annotationItems: mapPins) { pin in
                    MapAnnotation(coordinate: pin.coordinate) {
                        pinView(for: pin.kind)
                    }
                }
                .frame(height: 240)
                .cornerRadius(16)
                .padding(.horizontal, QervonSpacing.lg)
                .padding(.top, QervonSpacing.lg)

                QervonCard {
                    VStack(alignment: .leading, spacing: QervonSpacing.sm) {
                        Text(viewModel.order.status.displayName.uppercased())
                            .font(.system(size: 11, weight: .bold))
                            .foregroundColor(QervonColor.accent)
                        Text(viewModel.order.dropoff.label ?? "Teslim noktası")
                            .font(.system(size: 16, weight: .bold))
                            .foregroundColor(QervonColor.textPrimary)
                        Text(viewModel.order.fare.formatted)
                            .font(.system(size: 14, weight: .bold))
                            .foregroundColor(QervonColor.success)

                        if let deliveryNote = viewModel.order.deliveryNote {
                            Text("Not: \(deliveryNote)")
                                .font(.system(size: 12))
                                .foregroundColor(QervonColor.textSecondary)
                        }

                        if let eta = viewModel.eta {
                            Divider().background(QervonColor.border)
                            HStack {
                                Image(systemName: "clock.fill").foregroundColor(QervonColor.accent)
                                Text("Tahmini \(Int(eta.etaMinutes.rounded())) dakika")
                                    .font(.system(size: 13, weight: .semibold))
                                    .foregroundColor(QervonColor.textPrimary)
                            }
                        }
                    }
                    .frame(maxWidth: .infinity, alignment: .leading)
                }
                .padding(.horizontal, QervonSpacing.lg)

                if let errorMessage = viewModel.errorMessage {
                    Text(errorMessage)
                        .font(.system(size: 13))
                        .foregroundColor(QervonColor.danger)
                        .padding(.horizontal, QervonSpacing.lg)
                }

                VStack(spacing: QervonSpacing.sm) {
                    if viewModel.canCancel {
                        Button("Siparişi İptal Et") {
                            Task { _ = await viewModel.cancel() }
                        }
                        .buttonStyle(QervonButtonStyle(kind: .destructive, isEnabled: !viewModel.isCancelling))
                        .disabled(viewModel.isCancelling)
                    }
                    if viewModel.order.status == .delivered {
                        Button("Teslimatı Değerlendir") {
                            showingRating = true
                        }
                        .buttonStyle(QervonButtonStyle(kind: .primary))
                    }
                    Button("Destek Talebi Aç") {
                        showingSupport = true
                    }
                    .buttonStyle(QervonButtonStyle(kind: .secondary))
                }
                .padding(.horizontal, QervonSpacing.lg)
            }
            .padding(.bottom, QervonSpacing.xl)
        }
        .qervonScreenBackground()
        .navigationTitle("Sipariş Detayı")
        .onAppear { viewModel.onAppear() }
        .onDisappear { viewModel.onDisappear() }
        .onChange(of: viewModel.courierLocation) { location in
            if let location {
                withAnimation {
                    region.center = CLLocationCoordinate2D(latitude: location.latitude, longitude: location.longitude)
                }
            }
        }
        .sheet(isPresented: $showingRating) {
            RatingSheet { stars, comment in
                await viewModel.rate(stars: stars, comment: comment)
            }
        }
        .sheet(isPresented: $showingSupport) {
            SupportTicketSheet { subject, message in
                await viewModel.openSupportTicket(subject: subject, message: message)
            }
        }
    }

    private var mapPins: [OrderMapPin] {
        var pins = [
            OrderMapPin(
                id: "pickup",
                kind: .pickup,
                coordinate: CLLocationCoordinate2D(
                    latitude: viewModel.order.pickup.latitude, longitude: viewModel.order.pickup.longitude
                )
            ),
            OrderMapPin(
                id: "dropoff",
                kind: .dropoff,
                coordinate: CLLocationCoordinate2D(
                    latitude: viewModel.order.dropoff.latitude, longitude: viewModel.order.dropoff.longitude
                )
            ),
        ]
        if let location = viewModel.courierLocation {
            pins.append(OrderMapPin(
                id: "courier",
                kind: .courier,
                coordinate: CLLocationCoordinate2D(latitude: location.latitude, longitude: location.longitude)
            ))
        }
        return pins
    }

    private func pinView(for kind: OrderMapPin.Kind) -> some View {
        switch kind {
        case .pickup:
            return Image(systemName: "circle.fill").foregroundColor(QervonColor.accent)
        case .dropoff:
            return Image(systemName: "flag.fill").foregroundColor(QervonColor.danger)
        case .courier:
            return Image(systemName: "bicycle").foregroundColor(QervonColor.success)
        }
    }
}
