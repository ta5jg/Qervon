// =============================================================================
// File:           mobile/android/core/network/src/main/kotlin/com/qervon/core/network/HttpClientFactory.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Assembles the OkHttpClient (Bearer auth interceptor + refresh
//   authenticator + optional logging) and the Retrofit instance built on
//   top of it.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.network

import com.jakewharton.retrofit2.converter.kotlinx.serialization.asConverterFactory
import com.qervon.core.common.AuthTokenStore
import com.qervon.core.common.JsonConfig
import kotlinx.serialization.json.Json
import okhttp3.Interceptor
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.logging.HttpLoggingInterceptor
import retrofit2.Retrofit
import java.util.concurrent.TimeUnit

object HttpClientFactory {

    fun create(
        baseUrl: String,
        tokenStore: AuthTokenStore,
        enableLogging: Boolean,
        json: Json = JsonConfig.shared,
    ): QervonApiService {
        val normalizedBaseUrl = if (baseUrl.endsWith("/")) baseUrl else "$baseUrl/"

        val authInterceptor = Interceptor { chain ->
            val request = chain.request()
            val builder = request.newBuilder()
            tokenStore.currentTokens()?.let { tokens ->
                builder.header("Authorization", "Bearer ${tokens.accessToken}")
            }
            chain.proceed(builder.build())
        }

        val clientBuilder = OkHttpClient.Builder()
            .connectTimeout(15, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .writeTimeout(30, TimeUnit.SECONDS)
            .addInterceptor(authInterceptor)
            .authenticator(TokenAuthenticator(tokenStore, normalizedBaseUrl.removeSuffix("/")))

        if (enableLogging) {
            clientBuilder.addInterceptor(
                HttpLoggingInterceptor().apply { level = HttpLoggingInterceptor.Level.BASIC },
            )
        }

        val retrofit = Retrofit.Builder()
            .baseUrl(normalizedBaseUrl)
            .client(clientBuilder.build())
            .addConverterFactory(json.asConverterFactory("application/json".toMediaType()))
            .build()

        return retrofit.create(QervonApiService::class.java)
    }
}
