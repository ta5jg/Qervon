// =============================================================================
// File:           mobile/android/core/security/src/main/kotlin/com/qervon/core/security/EncryptedTokenStore.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   `AuthTokenStore` backed by `EncryptedSharedPreferences` (AES256-GCM
//   content, AES256-SIV keys) — the Android equivalent of the iOS
//   client's Keychain-backed `TokenStore`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.security

import android.content.Context
import android.content.SharedPreferences
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import com.qervon.core.common.AuthTokenStore
import com.qervon.core.common.JsonConfig
import com.qervon.core.common.model.AuthTokens
import kotlinx.serialization.SerializationException

private const val PREFS_FILE_PREFIX = "qervon_secure_tokens_"
private const val KEY_TOKENS_JSON = "tokens_json"

/**
 * @param appId Distinguishes the courier vs. customer app's token storage
 * when, in development, both apps happen to share a device user profile —
 * each app instance is expected to pass a stable, app-specific value
 * (e.g. its application ID).
 */
class EncryptedTokenStore(context: Context, appId: String) : AuthTokenStore {

    private val prefs: SharedPreferences by lazy {
        val masterKey = MasterKey.Builder(context)
            .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
            .build()
        EncryptedSharedPreferences.create(
            context,
            "$PREFS_FILE_PREFIX$appId",
            masterKey,
            EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
            EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
        )
    }

    @Volatile
    private var cached: AuthTokens? = null

    override fun currentTokens(): AuthTokens? {
        cached?.let { return it }
        val raw = prefs.getString(KEY_TOKENS_JSON, null) ?: return null
        return try {
            JsonConfig.shared.decodeFromString(AuthTokens.serializer(), raw).also { cached = it }
        } catch (_: SerializationException) {
            null
        }
    }

    override fun save(tokens: AuthTokens) {
        cached = tokens
        prefs.edit()
            .putString(KEY_TOKENS_JSON, JsonConfig.shared.encodeToString(AuthTokens.serializer(), tokens))
            .apply()
    }

    override fun clear() {
        cached = null
        prefs.edit().remove(KEY_TOKENS_JSON).apply()
    }
}
