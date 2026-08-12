// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonCore/APIError.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Error type shared across the app for backend/network failures. Mirrors
//   the `{status, title, detail}` JSON shape returned by the Qervon API
//   gateway's `ApiError`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation

/// The `{status, title, detail}` error body every Qervon API error responds
/// with (see `backend/apps/api-gateway/src/api_error.rs`).
public struct APIErrorBody: Decodable, Sendable {
    public let status: Int
    public let title: String
    public let detail: String
}

public enum APIError: Error, LocalizedError, Sendable {
    /// The backend returned a non-2xx response with a decodable error body.
    case server(status: Int, detail: String)
    /// The backend returned a non-2xx response we could not decode.
    case unexpectedStatus(Int)
    /// No access token is available and the caller required one.
    case unauthenticated
    /// The device has no usable network connection.
    case offline
    case decoding(Error)
    case transport(Error)

    public var errorDescription: String? {
        switch self {
        case let .server(_, detail):
            return detail
        case let .unexpectedStatus(status):
            return "Sunucu beklenmeyen bir durum kodu döndürdü: \(status)"
        case .unauthenticated:
            return "Oturum bulunamadı, lütfen tekrar giriş yapın."
        case .offline:
            return "İnternet bağlantısı yok."
        case .decoding:
            return "Sunucu yanıtı okunamadı."
        case let .transport(underlying):
            return underlying.localizedDescription
        }
    }
}
