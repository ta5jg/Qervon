// =============================================================================
// File:           mobile/ios/Features/QervonFeatures/Sources/ProofOfDeliveryFeature/SignaturePadView.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   A real finger-drawn signature capture surface. Renders the stroke to a
//   PNG and returns it base64-encoded, matching
//   `digital_signature_base64` on `POST /v1/courier/orders/{id}/deliver`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
import UIKit
import QervonDesignSystem

public struct SignaturePadView: View {
    @State private var strokes: [[CGPoint]] = []
    @State private var currentStroke: [CGPoint] = []
    private let onSave: (String) -> Void
    @Environment(\.dismiss) private var dismiss

    public init(onSave: @escaping (String) -> Void) {
        self.onSave = onSave
    }

    public var body: some View {
        VStack(spacing: QervonSpacing.md) {
            Text("Alıcı İmzası")
                .font(.system(size: 16, weight: .bold))
                .foregroundColor(QervonColor.textPrimary)

            Canvas { context, _ in
                for stroke in strokes + [currentStroke] {
                    guard stroke.count > 1 else { continue }
                    var path = Path()
                    path.move(to: stroke[0])
                    for point in stroke.dropFirst() {
                        path.addLine(to: point)
                    }
                    context.stroke(path, with: .color(.black), lineWidth: 3)
                }
            }
            .frame(height: 220)
            .background(Color.white)
            .cornerRadius(12)
            .gesture(
                DragGesture(minimumDistance: 0)
                    .onChanged { value in
                        currentStroke.append(value.location)
                    }
                    .onEnded { _ in
                        strokes.append(currentStroke)
                        currentStroke = []
                    }
            )

            HStack(spacing: QervonSpacing.sm) {
                Button("Temizle") {
                    strokes = []
                    currentStroke = []
                }
                .buttonStyle(QervonButtonStyle(kind: .secondary))

                Button("Kaydet") {
                    if let base64 = renderBase64PNG() {
                        onSave(base64)
                        dismiss()
                    }
                }
                .buttonStyle(QervonButtonStyle(kind: .primary, isEnabled: !strokes.isEmpty))
                .disabled(strokes.isEmpty)
            }
        }
        .padding(QervonSpacing.lg)
        .qervonScreenBackground()
        .presentationDetents([.medium])
    }

    private func renderBase64PNG() -> String? {
        let renderer = ImageRenderer(content:
            Canvas { context, _ in
                context.fill(Path(CGRect(x: 0, y: 0, width: 320, height: 220)), with: .color(.white))
                for stroke in strokes {
                    guard stroke.count > 1 else { continue }
                    var path = Path()
                    path.move(to: stroke[0])
                    for point in stroke.dropFirst() {
                        path.addLine(to: point)
                    }
                    context.stroke(path, with: .color(.black), lineWidth: 3)
                }
            }
            .frame(width: 320, height: 220)
        )
        guard let image = renderer.uiImage, let data = image.pngData() else { return nil }
        return data.base64EncodedString()
    }
}
