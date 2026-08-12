// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/ApiError.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Error type shared across the app for backend/network failures. Mirrors
//   the `{status, title, detail}` JSON shape returned by the Qervon API
//   gateway's `ApiError` (see `backend/apps/api-gateway/src/api_error.rs`).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common

import kotlinx.serialization.Serializable

@Serializable
data class ApiErrorBody(
    val status: Int,
    val title: String,
    val detail: String,
)

sealed class QervonApiException(message: String) : Exception(message) {
    /** The backend returned a non-2xx response with a decodable error body. */
    class Server(val status: Int, val detail: String) : QervonApiException(detail)

    /** The backend returned a non-2xx response we could not decode. */
    class UnexpectedStatus(val status: Int) :
        QervonApiException("Sunucu beklenmeyen bir durum kodu döndürdü: $status")

    /** No access token is available and the caller required one. */
    object Unauthenticated : QervonApiException("Oturum bulunamadı, lütfen tekrar giriş yapın.")

    class Decoding(cause: Throwable) : QervonApiException("Sunucu yanıtı okunamadı: ${cause.message}")

    class Transport(cause: Throwable) : QervonApiException(cause.message ?: "Ağ hatası oluştu.")
}
