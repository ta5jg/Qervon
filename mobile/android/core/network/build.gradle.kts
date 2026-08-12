// =============================================================================
// File:           mobile/android/core/network/build.gradle.kts
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Pure-Kotlin/JVM module: Retrofit + OkHttp HTTP client, Bearer-token
//   auth interceptor with 401 refresh-and-retry, and the typed
//   `QervonApi` service interfaces. Has no Android dependency (Retrofit
//   and OkHttp are plain JVM libraries), which keeps it fast to build and
//   unit-testable off-device — mirrors the iOS `QervonNetworking` SPM
//   target, which has the same platform-independence property.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

plugins {
    alias(libs.plugins.kotlin.jvm)
    alias(libs.plugins.kotlin.serialization)
}

kotlin {
    jvmToolchain(17)
}

dependencies {
    api(project(":core:common"))
    api(libs.kotlinx.serialization.json)
    implementation(libs.kotlinx.coroutines.core)
    api(libs.retrofit.core)
    implementation(libs.retrofit.kotlinx.serialization.converter)
    api(libs.okhttp.core)
    implementation(libs.okhttp.logging.interceptor)
    testImplementation(libs.junit)
}
