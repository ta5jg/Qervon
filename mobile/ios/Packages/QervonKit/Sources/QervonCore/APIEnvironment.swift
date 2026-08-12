// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/APIEnvironment.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Resolves the backend base URL. Defaults to the iOS Simulator's view of
//   the host Mac's loopback address; a real device cannot reach
//   `127.0.0.1` and must be pointed at the Mac's LAN IP from the Profile
//   screen's "Sunucu Adresi" field.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

public enum APIEnvironment {
    private static let overrideKey = "qervon.api_base_url_override"

    public static let defaultBaseURL = URL(string: "http://127.0.0.1:8080")!

    public static var baseURL: URL {
        if let override = UserDefaults.standard.string(forKey: overrideKey),
           let url = URL(string: override) {
            return url
        }
        return defaultBaseURL
    }

    public static func currentOverride() -> String? {
        UserDefaults.standard.string(forKey: overrideKey)
    }

    public static func setOverride(_ urlString: String?) {
        guard let urlString, !urlString.trimmingCharacters(in: .whitespaces).isEmpty else {
            UserDefaults.standard.removeObject(forKey: overrideKey)
            return
        }
        UserDefaults.standard.set(urlString, forKey: overrideKey)
    }
}
