// =============================================================================
// File:           mobile/ios/Packages/QervonKit/Sources/QervonNetworking/QervonAPI.swift
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Typed surface over every backend endpoint the Qervon Courier app uses.
//   Thin wrappers around `HTTPClient`; no business logic lives here.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

import Foundation
import QervonCore

public struct QervonAPI: Sendable {
    private let client: HTTPClient

    public init(client: HTTPClient) {
        self.client = client
    }

    // MARK: - Auth

    public func requestOtp(tenantSlug: String, phone: String) async throws -> OtpRequestResult {
        try await client.send(
            .post, "/v1/auth/otp/request",
            body: OtpRequestBody(tenantSlug: tenantSlug, phone: phone),
            authenticated: false
        )
    }

    public func verifyOtp(tenantSlug: String, phone: String, code: String) async throws -> AuthTokens {
        try await client.send(
            .post, "/v1/auth/otp/verify",
            body: OtpVerifyBody(tenantSlug: tenantSlug, phone: phone, code: code),
            authenticated: false
        )
    }

    public func login(email: String, password: String, tenantSlug: String) async throws -> AuthTokens {
        try await client.send(
            .post, "/v1/auth/login",
            body: LoginBody(email: email, password: password, tenantSlug: tenantSlug),
            authenticated: false
        )
    }

    public func logout(refreshToken: String) async throws {
        try await client.sendNoContent(
            .post, "/v1/auth/logout",
            body: RefreshBody(refreshToken: refreshToken),
            authenticated: false
        )
    }

    public func linkPhone(_ phone: String) async throws -> QervonUser {
        try await client.send(.post, "/v1/auth/phone", body: LinkPhoneBody(phone: phone))
    }

    /// Creates a `role=customer` account (and, if `tenantSlug` is given, a
    /// tenant membership). Returns no tokens — the caller must follow up
    /// with `login` (or `requestOtp`/`verifyOtp` once a phone is linked).
    public func register(
        email: String,
        displayName: String,
        password: String,
        tenantSlug: String?
    ) async throws {
        try await client.sendNoContent(
            .post, "/v1/auth/register",
            body: RegisterAccountBody(
                email: email, displayName: displayName, password: password, tenantSlug: tenantSlug
            ),
            authenticated: false
        )
    }

    // MARK: - Courier self-service

    public func getOwnCourier() async throws -> Courier {
        try await client.send(.get, "/v1/courier/me")
    }

    public func setAvailability(online: Bool) async throws -> Courier {
        try await client.send(.post, "/v1/courier/me/status", body: SetAvailabilityBody(online: online))
    }

    public func updateOwnLocation(_ body: UpdateLocationBody) async throws -> Courier {
        try await client.send(.post, "/v1/courier/me/location", body: body)
    }

    public func getPendingOffer() async throws -> PendingOffer? {
        try await client.send(.get, "/v1/courier/me/offer")
    }

    public func acceptOffer(orderId: UUID) async throws -> Order {
        try await client.send(.post, "/v1/courier/orders/\(orderId.uuidString)/accept", body: EmptyBody())
    }

    public func rejectOffer(orderId: UUID) async throws {
        try await client.sendNoContent(.post, "/v1/courier/orders/\(orderId.uuidString)/reject", body: EmptyBody())
    }

    public func listCourierOrders() async throws -> [Order] {
        try await client.send(.get, "/v1/courier/orders")
    }

    public func startTransit(orderId: UUID) async throws -> Order {
        try await client.send(.post, "/v1/courier/orders/\(orderId.uuidString)/pickup", body: EmptyBody())
    }

    public func deliverOrder(orderId: UUID, _ body: DeliverOrderBody) async throws -> Order {
        try await client.send(.post, "/v1/courier/orders/\(orderId.uuidString)/deliver", body: body)
    }

    /// Uploads a real delivery-proof photo (JPEG) for `orderId` to local
    /// server-side storage and returns the URL to pass as
    /// `DeliverOrderBody.photoEvidenceUrl`. See
    /// `backend/apps/api-gateway/src/http.rs`'s `upload_delivery_photo`.
    public func uploadDeliveryPhoto(orderId: UUID, jpegData: Data) async throws -> String {
        let response: UploadedFileResponse = try await client.uploadMultipart(
            "/v1/courier/orders/\(orderId.uuidString)/photo-evidence",
            fieldName: "photo",
            filename: "proof.jpg",
            mimeType: "image/jpeg",
            fileData: jpegData
        )
        return response.url
    }

    public func getWallet() async throws -> CourierWallet {
        try await client.send(.get, "/v1/courier/me/wallet")
    }

    public func getOwnRatings() async throws -> [CustomerRating] {
        try await client.send(.get, "/v1/courier/me/ratings")
    }

    // MARK: - Customer self-service

    public func getCustomerProfile() async throws -> CustomerProfile {
        try await client.send(.get, "/v1/customer/profile")
    }

    public func addAddress(
        label: String,
        latitude: Double,
        longitude: Double,
        fullAddress: String
    ) async throws -> CustomerProfile {
        try await client.send(
            .post, "/v1/customer/profile/addresses",
            body: AddAddressBody(label: label, latitude: latitude, longitude: longitude, fullAddress: fullAddress)
        )
    }

    public func removeAddress(id: UUID) async throws -> CustomerProfile {
        try await client.send(.delete, "/v1/customer/profile/addresses/\(id.uuidString)")
    }

    /// A non-binding fare estimate for a pickup/dropoff pair. The order is
    /// always charged the server's authoritative, freshly-recomputed fare
    /// at creation time — this is purely informational for the UI.
    public func getFareQuote(pickup: GeoLocation, dropoff: GeoLocation) async throws -> FareQuote {
        let path = "/v1/customer/fare-quote?pickup_latitude=\(pickup.latitude)" +
            "&pickup_longitude=\(pickup.longitude)&dropoff_latitude=\(dropoff.latitude)" +
            "&dropoff_longitude=\(dropoff.longitude)"
        return try await client.send(.get, path)
    }

    public func createOrder(_ body: CreateCustomerOrderBody) async throws -> Order {
        try await client.send(.post, "/v1/customer/orders", body: body)
    }

    public func listCustomerOrders() async throws -> [Order] {
        try await client.send(.get, "/v1/customer/orders")
    }

    public func cancelOrder(orderId: UUID) async throws -> Order {
        try await client.send(.post, "/v1/customer/orders/\(orderId.uuidString)/cancel", body: EmptyBody())
    }

    /// `nil` when there is no assigned courier yet or the order is not in a
    /// state where an ETA is meaningful — never an error for that case.
    public func getOrderEta(orderId: UUID) async throws -> EtaInfo? {
        try await client.send(.get, "/v1/customer/orders/\(orderId.uuidString)/eta")
    }

    /// A single, one-shot snapshot of the assigned courier's last reported
    /// location for this order (polled by the UI, not push-based). The
    /// backend responds with a 422 (not `null`) when there is no assigned
    /// courier yet or no location has been reported yet — both are normal,
    /// expected polling states here, not failures to surface.
    public func getOrderTracking(orderId: UUID) async throws -> LocationSnapshot? {
        do {
            return try await client.send(.get, "/v1/orders/\(orderId.uuidString)/tracking")
        } catch APIError.server {
            return nil
        }
    }

    public func rateOrder(orderId: UUID, stars: Int, comment: String?) async throws -> CustomerRating {
        try await client.send(
            .post, "/v1/customer/orders/\(orderId.uuidString)/rating",
            body: RateOrderBody(ratingStars: stars, comment: comment)
        )
    }

    public func createSupportTicket(
        orderId: UUID?,
        subject: String,
        message: String
    ) async throws -> SupportTicket {
        try await client.send(
            .post, "/v1/customer/support-tickets",
            body: OpenSupportTicketBody(orderId: orderId, subject: subject, message: message)
        )
    }

    public func listSupportTickets() async throws -> [SupportTicket] {
        try await client.send(.get, "/v1/customer/support-tickets")
    }

    public func listNotifications() async throws -> [AppNotification] {
        try await client.send(.get, "/v1/customer/notifications")
    }

    // MARK: - Native push registration

    public func registerPushDevice(platform: PushPlatform, deviceToken: String) async throws -> DevicePushToken {
        try await client.send(
            .post, "/v1/push/devices",
            body: RegisterPushDeviceBody(platform: platform.rawValue, deviceToken: deviceToken)
        )
    }

    public func listPushDevices() async throws -> [DevicePushToken] {
        try await client.send(.get, "/v1/push/devices")
    }

    public func deletePushDevice(id: UUID) async throws {
        try await client.sendNoContent(.delete, "/v1/push/devices/\(id.uuidString)")
    }
}
