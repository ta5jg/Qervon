// =============================================================================
// File:           mobile/android/app-customer/src/main/kotlin/com/qervon/android/customer/di/NetworkModule.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.android.customer.di

import android.content.Context
import com.qervon.android.customer.ApiEnvironment
import com.qervon.core.common.AuthTokenStore
import com.qervon.core.network.HttpClientFactory
import com.qervon.core.network.QervonApi
import com.qervon.core.network.QervonApiService
import com.qervon.core.security.AppPreferences
import com.qervon.core.security.EncryptedTokenStore
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import javax.inject.Singleton

@Module
@InstallIn(SingletonComponent::class)
object NetworkModule {

    @Provides
    @Singleton
    fun provideAuthTokenStore(@ApplicationContext context: Context): AuthTokenStore =
        EncryptedTokenStore(context, appId = "customer")

    @Provides
    @Singleton
    fun provideAppPreferences(@ApplicationContext context: Context): AppPreferences = AppPreferences(context)

    @Provides
    @Singleton
    fun provideQervonApiService(tokenStore: AuthTokenStore): QervonApiService =
        HttpClientFactory.create(baseUrl = ApiEnvironment.BASE_URL, tokenStore = tokenStore, enableLogging = true)

    @Provides
    @Singleton
    fun provideQervonApi(service: QervonApiService, tokenStore: AuthTokenStore): QervonApi = QervonApi(service, tokenStore)
}
