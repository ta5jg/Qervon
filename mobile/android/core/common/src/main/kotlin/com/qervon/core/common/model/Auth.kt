// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/model/Auth.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `AuthResponse` and decodes the backend's custom
//   `qv1.<payload>.<signature>` access token locally (read-only: the app
//   has no signing secret and cannot verify the signature, it only reads
//   the claims it was already issued over HTTPS — see
//   `backend/apps/api-gateway/src/auth.rs`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common.model

import com.qervon.core.common.JsonConfig
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import java.time.Instant
import java.util.Base64

@Serializable
data class AuthTokens(
    val accessToken: String,
    val refreshToken: String,
    val tokenType: String,
    val expiresInSeconds: Int,
)

@Serializable
enum class UserRole {
    @SerialName("customer") CUSTOMER,
    @SerialName("company") COMPANY,
    @SerialName("courier") COURIER,
    @SerialName("admin") ADMIN,
    @SerialName("super_admin") SUPER_ADMIN,
    @SerialName("operator") OPERATOR,
    @SerialName("dispatcher") DISPATCHER,
    @SerialName("fleet_manager") FLEET_MANAGER,
    @SerialName("support") SUPPORT,
}

/** Locally decoded claims from an access token, for display/expiry purposes. */
data class AccessTokenClaims(
    val subject: String,
    val tenantId: String,
    val role: UserRole,
    val expiresAt: Instant,
) {
    val isExpired: Boolean get() = !expiresAt.isAfter(Instant.now())
}

@Serializable
private data class RawAccessClaims(
    val subject: String,
    @SerialName("tenant_id") val tenantId: String,
    val role: UserRole,
    @SerialName("expires_at") val expiresAt: Long,
)

class AccessTokenDecodingException(message: String) : Exception(message)

/**
 * Decodes the `qv1.<payload>.<signature>` token format without verifying
 * the signature (the client has no signing secret; it trusts the token
 * because it just received it over HTTPS from the backend).
 */
fun decodeAccessTokenClaims(token: String): AccessTokenClaims {
    val parts = token.split(".")
    if (parts.size != 3 || parts[0] != "qv1") {
        throw AccessTokenDecodingException("malformed access token")
    }
    val payloadJson = try {
        String(Base64.getUrlDecoder().decode(padBase64Url(parts[1])))
    } catch (error: IllegalArgumentException) {
        throw AccessTokenDecodingException("malformed access token payload")
    }
    val raw = JsonConfig.shared.decodeFromString(RawAccessClaims.serializer(), payloadJson)
    return AccessTokenClaims(
        subject = raw.subject,
        tenantId = raw.tenantId,
        role = raw.role,
        expiresAt = Instant.ofEpochSecond(raw.expiresAt),
    )
}

/** The backend emits unpadded base64url (Rust's `URL_SAFE_NO_PAD`); Java's
 * decoder requires a length that's a multiple of 4, so padding is restored
 * before decoding. */
private fun padBase64Url(value: String): String {
    val remainder = value.length % 4
    return if (remainder == 0) value else value + "=".repeat(4 - remainder)
}
