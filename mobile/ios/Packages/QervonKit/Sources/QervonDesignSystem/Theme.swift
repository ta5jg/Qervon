// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonDesignSystem/Theme.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Shared dark-theme palette and typography, derived from the original
//   Courier prototype's colors so the visual identity carries over.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import SwiftUI
#if canImport(UIKit)
import UIKit
#endif

public enum QervonColor {
    public static let background = Color(red: 0.02, green: 0.03, blue: 0.07)
    public static let surface = Color(red: 0.06, green: 0.09, blue: 0.16)
    public static let accent = Color(red: 0.22, green: 0.74, blue: 0.97)
    public static let success = Color(red: 0.06, green: 0.72, blue: 0.51)
    public static let danger = Color(red: 0.93, green: 0.27, blue: 0.27)
    public static let warning = Color(red: 0.96, green: 0.62, blue: 0.11)
    public static let textPrimary = Color.white
    public static let textSecondary = Color.white.opacity(0.6)
    public static let border = Color.white.opacity(0.1)
}

public enum QervonSpacing {
    public static let xs: CGFloat = 4
    public static let sm: CGFloat = 8
    public static let md: CGFloat = 16
    public static let lg: CGFloat = 20
    public static let xl: CGFloat = 28
}

public struct QervonCard<Content: View>: View {
    private let content: Content
    private let accentBorder: Color?

    public init(accentBorder: Color? = nil, @ViewBuilder content: () -> Content) {
        self.accentBorder = accentBorder
        self.content = content()
    }

    public var body: some View {
        content
            .padding(QervonSpacing.md)
            .background(QervonColor.surface.opacity(0.85))
            .cornerRadius(16)
            .overlay(
                RoundedRectangle(cornerRadius: 16)
                    .stroke(accentBorder ?? QervonColor.border, lineWidth: 1)
            )
    }
}

public enum QervonButtonStyleKind {
    case primary
    case destructive
    case secondary
}

public struct QervonButtonStyle: ButtonStyle {
    let kind: QervonButtonStyleKind
    let isEnabled: Bool

    public init(kind: QervonButtonStyleKind = .primary, isEnabled: Bool = true) {
        self.kind = kind
        self.isEnabled = isEnabled
    }

    public func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.system(size: 15, weight: .bold))
            .foregroundColor(.white)
            .frame(maxWidth: .infinity)
            .padding(.vertical, 14)
            .background(background)
            .cornerRadius(14)
            .opacity(isEnabled ? (configuration.isPressed ? 0.85 : 1) : 0.4)
    }

    private var background: some View {
        let colors: [Color]
        switch kind {
        case .primary: colors = [QervonColor.success, QervonColor.success.opacity(0.8)]
        case .destructive: colors = [QervonColor.danger, QervonColor.danger.opacity(0.8)]
        case .secondary: colors = [QervonColor.surface, QervonColor.surface.opacity(0.8)]
        }
        return LinearGradient(colors: colors, startPoint: .leading, endPoint: .trailing)
    }
}

#if canImport(UIKit)
public struct QervonTextField: View {
    let title: String
    @Binding var text: String
    var isSecure: Bool
    var autocapitalize: Bool
    var keyboard: UIKeyboardType

    public init(
        title: String,
        text: Binding<String>,
        isSecure: Bool = false,
        autocapitalize: Bool = true,
        keyboard: UIKeyboardType = .default
    ) {
        self.title = title
        self._text = text
        self.isSecure = isSecure
        self.autocapitalize = autocapitalize
        self.keyboard = keyboard
    }

    public var body: some View {
        Group {
            if isSecure {
                SecureField(title, text: $text)
            } else {
                TextField(title, text: $text)
                    .keyboardType(keyboard)
                    .textInputAutocapitalization(autocapitalize ? .sentences : .never)
                    .autocorrectionDisabled(!autocapitalize)
            }
        }
        .padding(12)
        .background(QervonColor.background.opacity(0.6))
        .cornerRadius(10)
        .overlay(RoundedRectangle(cornerRadius: 10).stroke(QervonColor.border, lineWidth: 1))
        .foregroundColor(QervonColor.textPrimary)
    }
}
#endif

public struct QervonScreenBackground: ViewModifier {
    public init() {}

    public func body(content: Content) -> some View {
        ZStack {
            QervonColor.background.ignoresSafeArea()
            content
        }
    }
}

public extension View {
    func qervonScreenBackground() -> some View {
        modifier(QervonScreenBackground())
    }
}
