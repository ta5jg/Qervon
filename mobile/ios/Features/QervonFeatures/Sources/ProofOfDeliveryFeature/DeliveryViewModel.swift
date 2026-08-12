// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/ProofOfDeliveryFeature/DeliveryViewModel.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import UIKit
import QervonCore
import QervonNetworking

@MainActor
public final class DeliveryViewModel: ObservableObject {
    public let order: Order

    @Published public var recipientName = ""
    @Published public var qrBarcodeVerified = false
    @Published public var scannedCode: String?
    @Published public var signatureBase64: String?
    @Published public var localPhoto: UIImage?
    @Published public var paymentCollected = false
    @Published public private(set) var isSubmitting = false
    @Published public var errorMessage: String?

    private let api: QervonAPI

    public init(order: Order, api: QervonAPI) {
        self.order = order
        self.api = api
    }

    public var requiresPaymentConfirmation: Bool {
        order.paymentMethod == .cash
    }

    /// The backend requires at least one real proof (QR/barcode verified,
    /// a signature, or photo evidence). A photo alone is not required
    /// client-side because its upload (see `submit()`) can fail without
    /// blocking delivery when a QR/signature proof already exists; a photo
    /// that *does* upload successfully still gets sent as
    /// `photoEvidenceUrl` regardless of which of these is what satisfied
    /// this check.
    public var canSubmit: Bool {
        !recipientName.trimmingCharacters(in: .whitespaces).isEmpty
            && (qrBarcodeVerified || signatureBase64 != nil)
    }

    public func handleScannedCode(_ code: String) {
        scannedCode = code
        qrBarcodeVerified = true
    }

    public func submit() async -> Order? {
        guard canSubmit else { return nil }
        isSubmitting = true
        errorMessage = nil
        defer { isSubmitting = false }

        var photoEvidenceUrl: String?
        if let localPhoto {
            _ = LocalDeliveryEvidenceStore.save(localPhoto, forOrderId: order.id.uuidString)
            if let jpegData = localPhoto.jpegData(compressionQuality: 0.7) {
                do {
                    photoEvidenceUrl = try await api.uploadDeliveryPhoto(
                        orderId: order.id, jpegData: jpegData
                    )
                } catch {
                    // The photo is still kept locally (above); a failed
                    // upload should not block delivery confirmation itself
                    // when a QR/signature proof already satisfies
                    // `canSubmit`.
                }
            }
        }
        let body = DeliverOrderBody(
            recipientName: recipientName,
            qrBarcodeVerified: qrBarcodeVerified,
            digitalSignatureBase64: signatureBase64,
            photoEvidenceUrl: photoEvidenceUrl,
            paymentCollected: requiresPaymentConfirmation ? paymentCollected : false
        )
        do {
            return try await api.deliverOrder(orderId: order.id, body)
        } catch {
            errorMessage = error.localizedDescription
            return nil
        }
    }
}
