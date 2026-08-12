// =============================================================================
// File:           mobile/android/settings.gradle.kts
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-05
// Version:        0.2.0
//
// Description:
//   Gradle module registry for the Qervon Android apps (Courier +
//   Customer). Two real application modules share the `core:*`
//   infrastructure modules and the `feature:auth` module; everything else
//   under `feature:*` is app-specific.
//
// Specification:
//   QAS-000004, QAS-000005, QAS-000007, QES-000003, QES-000004.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}

dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "qervon-android"

include(
    ":app-courier",
    ":app-customer",
    ":core:common",
    ":core:network",
    ":core:security",
    ":core:location",
    ":core:designsystem",
    ":feature:auth",
    ":feature:dispatch",
    ":feature:orders",
    ":feature:proof",
    ":feature:earnings",
    ":feature:profile",
    ":feature:addressbook",
    ":feature:customerorder",
    ":feature:customerprofile",
)
