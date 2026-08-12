// =============================================================================
// File:           mobile/android/core/network/src/main/kotlin/com/qervon/core/network/Requests.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Request/response wire DTOs that only exist at the HTTP boundary
//   (mirrors the private structs declared inline in
//   `backend/apps/api-gateway/src/http.rs`, e.g. `LoginRequest`,
//   `AuthResponse`, `OtpRequestRequest`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.network

import com.qervon.core.common.model.Address
import kotlinx.serialization.Serializable

@Serializable
data class LoginBody(val email: String, val password: String, val tenantSlug: String)

@Serializable
data class RegisterAccountBody(
    val email: String,
    val displayName: String,
    val password: String,
    val tenantSlug: String? = null,
)

@Serializable
data class RefreshBody(val refreshToken: String)

@Serializable
data class OtpRequestBody(val tenantSlug: String, val phone: String)

@Serializable
data class OtpRequestResponseBody(val status: String, val devCode: String? = null)

@Serializable
data class OtpVerifyBody(val tenantSlug: String, val phone: String, val code: String)

@Serializable
data class SetPhoneBody(val phone: String)

@Serializable
data class AuthResponseBody(
    val accessToken: String,
    val refreshToken: String,
    val tokenType: String,
    val expiresInSeconds: Long,
)

@Serializable
data class SetCourierAvailabilityBody(val online: Boolean)

@Serializable
data class UpdateLocationBody(
    val latitude: Double,
    val longitude: Double,
    val speedKmh: Double? = null,
    val batteryPct: Int? = null,
)

@Serializable
data class CompleteDeliveryBody(
    val recipientName: String,
    val qrBarcodeVerified: Boolean,
    val digitalSignatureBase64: String? = null,
    val photoEvidenceUrl: String? = null,
    val paymentCollected: Boolean = false,
)

@Serializable
data class UploadedFileResponseBody(val url: String)

@Serializable
data class CreateCustomerOrderBody(
    val pickup: Address,
    val dropoff: Address,
    val couponCode: String? = null,
    val paymentMethod: String? = null,
    val deliveryNote: String? = null,
    val contactPhone: String? = null,
)

@Serializable
data class CreateCustomerAddressBody(
    val label: String,
    val latitude: Double,
    val longitude: Double,
    val fullAddress: String,
)

@Serializable
data class RateOrderBody(val ratingStars: Int, val comment: String? = null)

@Serializable
data class OpenSupportTicketBody(
    val orderId: String? = null,
    val subject: String,
    val message: String,
)
