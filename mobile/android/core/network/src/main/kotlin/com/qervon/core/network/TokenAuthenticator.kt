// =============================================================================
// File:           mobile/android/core/network/src/main/kotlin/com/qervon/core/network/TokenAuthenticator.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   OkHttp `Authenticator` that transparently refreshes an expired access
//   token on a 401 and retries the original request exactly once. Guards
//   against concurrent refreshes (two in-flight requests hitting 401 at
//   the same time) and infinite retry loops.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.network

import com.qervon.core.common.AuthTokenStore
import com.qervon.core.common.JsonConfig
import com.qervon.core.common.model.AuthTokens
import okhttp3.Authenticator
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import okhttp3.Response
import okhttp3.Route
import java.io.IOException

private const val MAX_RETRY_ATTEMPTS = 2

class TokenAuthenticator(
    private val tokenStore: AuthTokenStore,
    private val baseUrl: String,
) : Authenticator {

    /** A plain client with no authenticator/interceptor of its own —
     * refreshing must never itself trigger another 401 -> refresh cycle. */
    private val refreshClient = OkHttpClient.Builder().build()

    override fun authenticate(route: Route?, response: Response): Request? {
        if (responseCount(response) >= MAX_RETRY_ATTEMPTS) {
            return null
        }
        val requestToken = response.request.header("Authorization")?.removePrefix("Bearer ")

        synchronized(this) {
            val latest = tokenStore.currentTokens() ?: return null
            if (requestToken != null && requestToken != latest.accessToken) {
                // Another thread already refreshed while we were waiting on the lock.
                return response.request.newBuilder()
                    .header("Authorization", "Bearer ${latest.accessToken}")
                    .build()
            }
            val refreshed = performRefresh(latest.refreshToken) ?: run {
                tokenStore.clear()
                return null
            }
            tokenStore.save(refreshed)
            return response.request.newBuilder()
                .header("Authorization", "Bearer ${refreshed.accessToken}")
                .build()
        }
    }

    private fun performRefresh(refreshToken: String): AuthTokens? {
        val body = JsonConfig.shared.encodeToString(
            RefreshBody.serializer(),
            RefreshBody(refreshToken = refreshToken),
        )
        val request = Request.Builder()
            .url("$baseUrl/v1/auth/refresh")
            .post(body.toRequestBody("application/json".toMediaType()))
            .build()
        return try {
            refreshClient.newCall(request).execute().use { response ->
                if (!response.isSuccessful) return null
                val json = response.body?.string() ?: return null
                val parsed = JsonConfig.shared.decodeFromString(AuthResponseBody.serializer(), json)
                AuthTokens(
                    accessToken = parsed.accessToken,
                    refreshToken = parsed.refreshToken,
                    tokenType = parsed.tokenType,
                    expiresInSeconds = parsed.expiresInSeconds.toInt(),
                )
            }
        } catch (_: IOException) {
            null
        }
    }

    private fun responseCount(response: Response): Int {
        var result = 1
        var prior = response.priorResponse
        while (prior != null) {
            result += 1
            prior = prior.priorResponse
        }
        return result
    }
}
