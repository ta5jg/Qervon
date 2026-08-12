// =============================================================================
// File:           mobile/android/app-courier/src/main/kotlin/com/qervon/android/courier/MainActivity.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Extends `FragmentActivity` (rather than plain `ComponentActivity`)
//   specifically because `BiometricPrompt` requires a `FragmentActivity`
//   host — see `feature:auth`'s `BiometricLockScreen`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.android.courier

import android.os.Bundle
import androidx.activity.compose.setContent
import androidx.fragment.app.FragmentActivity
import com.qervon.core.designsystem.QervonTheme
import dagger.hilt.android.AndroidEntryPoint

@AndroidEntryPoint
class MainActivity : FragmentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            QervonTheme {
                CourierApp()
            }
        }
    }
}
