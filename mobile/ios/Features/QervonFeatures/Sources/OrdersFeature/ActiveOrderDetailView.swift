// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/OrdersFeature/ActiveOrderDetailView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   The courier's currently assigned job: pickup (if not yet in transit),
//   navigation to whichever leg is next, and delivery. There is at most one
//   active job at a time (a busy courier cannot be offered another).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import QervonCore
import QervonNetworking
import QervonDesignSystem
import MapsFeature
import ProofOfDeliveryFeature

public struct ActiveOrderDetailView: View {
    @State private var order: Order
    let api: QervonAPI
    let onCompleted: () -> Void

    @State private var showingNavigationSheet = false
    @State private var showingPickupSheet = false
    @State private var showingDeliverSheet = false

    public init(order: Order, api: QervonAPI, onCompleted: @escaping () -> Void) {
        self._order = State(initialValue: order)
        self.api = api
        self.onCompleted = onCompleted
    }

    public var body: some View {
        ScrollView {
            VStack(spacing: QervonSpacing.lg) {
                QervonCard {
                    VStack(alignment: .leading, spacing: QervonSpacing.sm) {
                        Text(order.status.displayName.uppercased())
                            .font(.system(size: 11, weight: .bold))
                            .foregroundColor(QervonColor.accent)

                        addressRow(icon: "circle.fill", label: order.pickup.label ?? "Alım noktası", isActive: order.status == .courierAssigned)
                        addressRow(icon: "flag.fill", label: order.dropoff.label ?? "Teslim noktası", isActive: order.status == .inTransit)

                        Divider().background(QervonColor.border)

                        HStack {
                            Text(order.fare.formatted)
                                .font(.system(size: 16, weight: .bold))
                                .foregroundColor(QervonColor.success)
                            Spacer()
                            if let method = order.paymentMethod {
                                Text(method.displayName)
                                    .font(.system(size: 12, weight: .semibold))
                                    .foregroundColor(QervonColor.textSecondary)
                            }
                        }
                    }
                }
                .padding(.horizontal, QervonSpacing.lg)

                VStack(spacing: QervonSpacing.sm) {
                    Button("Navigasyona Başla") {
                        showingNavigationSheet = true
                    }
                    .buttonStyle(QervonButtonStyle(kind: .secondary))

                    if order.status == .courierAssigned {
                        Button("Teslim Al") {
                            showingPickupSheet = true
                        }
                        .buttonStyle(QervonButtonStyle(kind: .primary))
                    } else if order.status == .inTransit {
                        Button("Teslim Et") {
                            showingDeliverSheet = true
                        }
                        .buttonStyle(QervonButtonStyle(kind: .primary))
                    }
                }
                .padding(.horizontal, QervonSpacing.lg)
            }
            .padding(.vertical, QervonSpacing.lg)
        }
        .qervonScreenBackground()
        .navigationTitle("Aktif İş")
        .sheet(isPresented: $showingNavigationSheet) {
            NavigationPickerSheet(destination: currentDestination, label: currentLabel)
        }
        .sheet(isPresented: $showingPickupSheet) {
            PickupView(order: order, api: api) { updated in
                order = updated
            }
        }
        .sheet(isPresented: $showingDeliverSheet) {
            DeliverView(order: order, api: api) { _ in
                onCompleted()
            }
        }
    }

    private var currentDestination: GeoLocation {
        order.status == .courierAssigned
            ? GeoLocation(latitude: order.pickup.latitude, longitude: order.pickup.longitude)
            : GeoLocation(latitude: order.dropoff.latitude, longitude: order.dropoff.longitude)
    }

    private var currentLabel: String {
        order.status == .courierAssigned
            ? (order.pickup.label ?? "Alım noktası")
            : (order.dropoff.label ?? "Teslim noktası")
    }

    private func addressRow(icon: String, label: String, isActive: Bool) -> some View {
        HStack(spacing: QervonSpacing.sm) {
            Image(systemName: icon)
                .foregroundColor(isActive ? QervonColor.accent : QervonColor.textSecondary)
                .font(.system(size: 12))
            Text(label)
                .font(.system(size: 14, weight: isActive ? .bold : .regular))
                .foregroundColor(isActive ? QervonColor.textPrimary : QervonColor.textSecondary)
        }
    }
}
