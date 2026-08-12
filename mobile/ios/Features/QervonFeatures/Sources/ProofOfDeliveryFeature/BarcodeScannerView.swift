// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/ProofOfDeliveryFeature/BarcodeScannerView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Real QR/barcode scanning via VisionKit's `DataScannerViewController`
//   (iOS 16+). `DataScannerViewController.isSupported` is `false` on the
//   Simulator and on devices without the required camera — callers must
//   check that and fall back to a manual confirmation toggle instead of
//   pretending to scan (see ProofOfDeliverySection.swift).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import VisionKit
import QervonDesignSystem

public struct BarcodeScannerView: UIViewControllerRepresentable {
    let onScan: (String) -> Void

    public init(onScan: @escaping (String) -> Void) {
        self.onScan = onScan
    }

    public func makeUIViewController(context: Context) -> DataScannerViewController {
        let controller = DataScannerViewController(
            recognizedDataTypes: [.barcode()],
            qualityLevel: .balanced,
            isHighFrameRateTrackingEnabled: false,
            isPinchToZoomEnabled: true,
            isGuidanceEnabled: true,
            isHighlightingEnabled: true
        )
        controller.delegate = context.coordinator
        return controller
    }

    public func updateUIViewController(_ controller: DataScannerViewController, context: Context) {
        try? controller.startScanning()
    }

    public func makeCoordinator() -> Coordinator {
        Coordinator(onScan: onScan)
    }

    public final class Coordinator: NSObject, DataScannerViewControllerDelegate {
        let onScan: (String) -> Void

        init(onScan: @escaping (String) -> Void) {
            self.onScan = onScan
        }

        public func dataScanner(_ dataScanner: DataScannerViewController, didAdd items: [RecognizedItem], allItems: [RecognizedItem]) {
            for item in items {
                if case let .barcode(barcode) = item {
                    onScan(barcode.payloadStringValue ?? "")
                    dataScanner.stopScanning()
                    return
                }
            }
        }
    }
}

public struct BarcodeScannerSheet: View {
    let onScan: (String) -> Void
    @Environment(\.dismiss) private var dismiss

    public init(onScan: @escaping (String) -> Void) {
        self.onScan = onScan
    }

    public var body: some View {
        VStack(spacing: 0) {
            if DataScannerViewController.isSupported && DataScannerViewController.isAvailable {
                BarcodeScannerView { code in
                    onScan(code)
                    dismiss()
                }
            } else {
                VStack(spacing: QervonSpacing.md) {
                    Image(systemName: "camera.metering.unknown")
                        .font(.system(size: 40))
                        .foregroundColor(QervonColor.textSecondary)
                    Text("Bu cihazda barkod tarayıcı desteklenmiyor (Simülatörde kamera yoktur).")
                        .multilineTextAlignment(.center)
                        .font(.system(size: 13))
                        .foregroundColor(QervonColor.textSecondary)
                    Button("Kapat") { dismiss() }
                        .buttonStyle(QervonButtonStyle(kind: .secondary))
                }
                .padding(QervonSpacing.lg)
                .qervonScreenBackground()
            }
        }
    }
}
