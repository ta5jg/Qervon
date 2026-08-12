// =============================================================================
// File:           mobile/android/core/common/build.gradle.kts
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Pure-Kotlin/JVM module: shared DTOs mirroring backend JSON shapes,
//   ApiError, and date/money formatting helpers. No Android dependency —
//   these are plain data classes, so a JVM module builds faster and stays
//   trivially unit-testable.
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
    implementation(libs.kotlinx.serialization.json)
    testImplementation(libs.junit)
}
