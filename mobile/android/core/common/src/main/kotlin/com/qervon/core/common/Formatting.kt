// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/Formatting.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Shared date/time display formatting used across feature screens.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common

import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import java.time.temporal.ChronoUnit
import java.util.Locale

object QervonFormat {
    private val zone: ZoneId = ZoneId.systemDefault()
    private val timeFormatter = DateTimeFormatter.ofPattern("HH:mm", Locale("tr", "TR"))
    private val dayAndTimeFormatter = DateTimeFormatter.ofPattern("d MMM, HH:mm", Locale("tr", "TR"))

    fun time(instant: Instant): String = timeFormatter.withZone(zone).format(instant)

    fun dayAndTime(instant: Instant): String = dayAndTimeFormatter.withZone(zone).format(instant)

    fun startOfDay(now: Instant = Instant.now()): Instant =
        now.atZone(zone).toLocalDate().atStartOfDay(zone).toInstant()

    fun startOfWeek(now: Instant = Instant.now()): Instant {
        val date = now.atZone(zone).toLocalDate()
        val daysFromMonday = (date.dayOfWeek.value - 1).toLong() // Monday = 1
        return date.minus(daysFromMonday, ChronoUnit.DAYS).atStartOfDay(zone).toInstant()
    }

    fun startOfMonth(now: Instant = Instant.now()): Instant {
        val date = now.atZone(zone).toLocalDate().withDayOfMonth(1)
        return date.atStartOfDay(zone).toInstant()
    }
}
