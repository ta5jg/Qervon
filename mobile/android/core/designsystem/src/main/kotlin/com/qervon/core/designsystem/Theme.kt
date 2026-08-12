// =============================================================================
// File:           mobile/android/core/designsystem/src/main/kotlin/com/qervon/core/designsystem/Theme.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Root Material3 `MaterialTheme` wrapper shared by both apps.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.designsystem

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp

private val lightScheme = lightColorScheme(
    primary = QervonColors.Primary,
    secondary = QervonColors.Secondary,
    background = QervonColors.Background,
    surface = QervonColors.Surface,
    error = QervonColors.Danger,
)

private val darkScheme = darkColorScheme(
    primary = QervonColors.Primary,
    secondary = QervonColors.Secondary,
    background = QervonColors.BackgroundDark,
    surface = QervonColors.SurfaceDark,
    error = QervonColors.Danger,
)

private val qervonTypography = Typography(
    headlineMedium = TextStyle(fontWeight = FontWeight.Bold, fontSize = 24.sp),
    titleLarge = TextStyle(fontWeight = FontWeight.SemiBold, fontSize = 20.sp),
    titleMedium = TextStyle(fontWeight = FontWeight.SemiBold, fontSize = 16.sp),
    bodyLarge = TextStyle(fontSize = 16.sp),
    bodyMedium = TextStyle(fontSize = 14.sp),
    labelSmall = TextStyle(fontSize = 12.sp),
)

@Composable
fun QervonTheme(
    useDarkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit,
) {
    MaterialTheme(
        colorScheme = if (useDarkTheme) darkScheme else lightScheme,
        typography = qervonTypography,
        content = content,
    )
}
