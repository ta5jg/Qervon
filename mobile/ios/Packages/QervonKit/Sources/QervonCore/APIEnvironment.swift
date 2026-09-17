// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/APIEnvironment.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Resolves the backend base URL. Every build defaults to Qervon's live
//   HTTPS API so an Xcode-installed Debug build behaves like the beta app on
//   a physical device. Developers can still opt in to a local backend from
//   the Profile screen's "Sunucu Adresi" field.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public enum APIEnvironment {
    private static let overrideKey = "qervon.api_base_url_override"

    public static let defaultBaseURL = URL(string: "https://qervon.io")!

    public static var baseURL: URL {
        if let override = currentOverride(), let url = validatedURL(override) {
            return url
        }
        return defaultBaseURL
    }

    public static func currentOverride() -> String? {
        UserDefaults.standard.string(forKey: overrideKey)
    }

    public static func setOverride(_ urlString: String?) {
        guard let urlString else {
            UserDefaults.standard.removeObject(forKey: overrideKey)
            return
        }

        let trimmed = urlString.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else {
            UserDefaults.standard.removeObject(forKey: overrideKey)
            return
        }
        guard validatedURL(trimmed) != nil else { return }
        UserDefaults.standard.set(trimmed, forKey: overrideKey)
    }

    private static func validatedURL(_ value: String) -> URL? {
        guard let url = URL(string: value),
              let scheme = url.scheme?.lowercased(),
              scheme == "https" || scheme == "http",
              url.host != nil else {
            return nil
        }
        return url
    }
}
