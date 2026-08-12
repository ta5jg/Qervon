// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonNetworking/HTTPClient.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Low-level async/await HTTP client for the Qervon API gateway. Injects
//   the Bearer access token, retries exactly once after a transparent
//   refresh on 401, and maps every non-2xx response to `APIError`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import QervonCore

public enum HTTPMethod: String {
    case get = "GET"
    case post = "POST"
    case delete = "DELETE"
}

/// Thrown internally to trigger the single refresh-and-retry path; never
/// escapes `HTTPClient`.
private struct Unauthorized: Error {}

public actor HTTPClient {
    public let baseURL: URL
    private let session: URLSession
    private let tokenStore: AuthTokenStoring
    private let decoder = QervonJSON.makeDecoder()
    private let encoder = QervonJSON.makeEncoder()

    public init(baseURL: URL, tokenStore: AuthTokenStoring, session: URLSession = .shared) {
        self.baseURL = baseURL
        self.tokenStore = tokenStore
        self.session = session
    }

    /// A request with a JSON body, decoding a JSON response.
    public func send<Body: Encodable, Response: Decodable>(
        _ method: HTTPMethod,
        _ path: String,
        body: Body,
        authenticated: Bool = true
    ) async throws -> Response {
        let data = try await sendRaw(method, path, bodyData: try encoder.encode(body), authenticated: authenticated)
        return try decodeResponse(data)
    }

    /// A request with no body, decoding a JSON response.
    public func send<Response: Decodable>(
        _ method: HTTPMethod,
        _ path: String,
        authenticated: Bool = true
    ) async throws -> Response {
        let data = try await sendRaw(method, path, bodyData: nil, authenticated: authenticated)
        return try decodeResponse(data)
    }

    /// A request with a JSON body and no meaningful response body (e.g. 204).
    public func sendNoContent<Body: Encodable>(
        _ method: HTTPMethod,
        _ path: String,
        body: Body,
        authenticated: Bool = true
    ) async throws {
        _ = try await sendRaw(method, path, bodyData: try encoder.encode(body), authenticated: authenticated)
    }

    /// A request with no body and no meaningful response body (e.g. 204).
    public func sendNoContent(
        _ method: HTTPMethod,
        _ path: String,
        authenticated: Bool = true
    ) async throws {
        _ = try await sendRaw(method, path, bodyData: nil, authenticated: authenticated)
    }

    /// Uploads `fileData` as a single-field `multipart/form-data` body
    /// (used by `QervonAPI.uploadDeliveryPhoto`) and decodes a JSON
    /// response. Unlike `send`, the request body is not JSON — the
    /// server-side endpoint this targets expects a real file upload, not a
    /// JSON-encoded description of one.
    public func uploadMultipart<Response: Decodable>(
        _ path: String,
        fieldName: String,
        filename: String,
        mimeType: String,
        fileData: Data
    ) async throws -> Response {
        let boundary = "QervonBoundary-\(UUID().uuidString)"
        var body = Data()
        body.append("--\(boundary)\r\n".utf8Data)
        body.append(
            "Content-Disposition: form-data; name=\"\(fieldName)\"; filename=\"\(filename)\"\r\n"
                .utf8Data
        )
        body.append("Content-Type: \(mimeType)\r\n\r\n".utf8Data)
        body.append(fileData)
        body.append("\r\n--\(boundary)--\r\n".utf8Data)

        let data = try await sendRaw(
            .post,
            path,
            bodyData: body,
            authenticated: true,
            contentType: "multipart/form-data; boundary=\(boundary)"
        )
        return try decodeResponse(data)
    }

    private func decodeResponse<Response: Decodable>(_ data: Data) throws -> Response {
        // A `null` JSON body decodes to any Optional type; `Response` here
        // is expected to be `T?` when the endpoint can return no content.
        do {
            return try decoder.decode(Response.self, from: data)
        } catch {
            throw APIError.decoding(error)
        }
    }

    private func sendRaw(
        _ method: HTTPMethod,
        _ path: String,
        bodyData: Data?,
        authenticated: Bool,
        contentType: String = "application/json"
    ) async throws -> Data {
        do {
            return try await performOnce(
                method, path, bodyData: bodyData, authenticated: authenticated, contentType: contentType
            )
        } catch is Unauthorized {
            try await refreshTokens()
            return try await performOnce(
                method, path, bodyData: bodyData, authenticated: authenticated, contentType: contentType
            )
        }
    }

    private func performOnce(
        _ method: HTTPMethod,
        _ path: String,
        bodyData: Data?,
        authenticated: Bool,
        contentType: String = "application/json"
    ) async throws -> Data {
        guard let url = URL(string: path, relativeTo: baseURL) else {
            throw APIError.transport(URLError(.badURL))
        }
        var request = URLRequest(url: url)
        request.httpMethod = method.rawValue
        request.setValue(contentType, forHTTPHeaderField: "Content-Type")
        if let bodyData {
            request.httpBody = bodyData
        }
        if authenticated {
            guard let tokens = tokenStore.currentTokens() else {
                throw APIError.unauthenticated
            }
            request.setValue("Bearer \(tokens.accessToken)", forHTTPHeaderField: "Authorization")
        }

        let (data, response): (Data, URLResponse)
        do {
            (data, response) = try await session.data(for: request)
        } catch {
            throw APIError.transport(error)
        }
        guard let httpResponse = response as? HTTPURLResponse else {
            throw APIError.unexpectedStatus(-1)
        }
        switch httpResponse.statusCode {
        case 200..<300:
            return data
        case 401 where authenticated:
            throw Unauthorized()
        default:
            if let body = try? decoder.decode(APIErrorBody.self, from: data) {
                throw APIError.server(status: body.status, detail: body.detail)
            }
            throw APIError.unexpectedStatus(httpResponse.statusCode)
        }
    }

    private func refreshTokens() async throws {
        guard let tokens = tokenStore.currentTokens() else {
            throw APIError.unauthenticated
        }
        let refreshed: AuthTokens
        do {
            refreshed = try await performOnce(
                .post,
                "/v1/auth/refresh",
                bodyData: try encoder.encode(["refresh_token": tokens.refreshToken]),
                authenticated: false
            ).decoded(as: AuthTokens.self, using: decoder)
        } catch {
            tokenStore.clear()
            throw error
        }
        try tokenStore.save(tokens: refreshed)
    }
}

private extension Data {
    func decoded<T: Decodable>(as type: T.Type, using decoder: JSONDecoder) throws -> T {
        do {
            return try decoder.decode(T.self, from: self)
        } catch {
            throw APIError.decoding(error)
        }
    }
}

private extension String {
    var utf8Data: Data { Data(utf8) }
}
