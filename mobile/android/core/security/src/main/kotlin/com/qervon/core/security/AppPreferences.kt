// =============================================================================
// File:           mobile/android/core/security/src/main/kotlin/com/qervon/core/security/AppPreferences.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Plain (non-encrypted) app settings — biometric-lock opt-in, last
//   used tenant slug for convenience on the login screen, etc. Nothing
//   sensitive is stored here; tokens always go through
//   `EncryptedTokenStore`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.security

import android.content.Context

class AppPreferences(context: Context) {
    private val prefs = context.getSharedPreferences("qervon_app_prefs", Context.MODE_PRIVATE)

    var biometricLockEnabled: Boolean
        get() = prefs.getBoolean(KEY_BIOMETRIC_LOCK, false)
        set(value) = prefs.edit().putBoolean(KEY_BIOMETRIC_LOCK, value).apply()

    var lastTenantSlug: String?
        get() = prefs.getString(KEY_LAST_TENANT_SLUG, null)
        set(value) = prefs.edit().putString(KEY_LAST_TENANT_SLUG, value).apply()

    var courierOnlineOnAppStart: Boolean
        get() = prefs.getBoolean(KEY_COURIER_ONLINE_ON_START, false)
        set(value) = prefs.edit().putBoolean(KEY_COURIER_ONLINE_ON_START, value).apply()

    private companion object {
        const val KEY_BIOMETRIC_LOCK = "biometric_lock_enabled"
        const val KEY_LAST_TENANT_SLUG = "last_tenant_slug"
        const val KEY_COURIER_ONLINE_ON_START = "courier_online_on_start"
    }
}
