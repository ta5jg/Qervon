// =============================================================================
// File:           mobile/android/core/common/src/main/kotlin/com/qervon/core/common/model/AppNotification.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Mirrors `qervon_domain::Notification` (`GET /v1/customer/notifications`).
//   Named `AppNotification` to avoid clashing with Android's own
//   `android.app.Notification`.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common.model

import com.qervon.core.common.InstantSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import java.time.Instant

@Serializable
enum class NotificationChannel {
    @SerialName("push") PUSH,
    @SerialName("sms") SMS,
    @SerialName("email") EMAIL,
    @SerialName("whatsapp") WHATSAPP,
}

@Serializable
enum class NotificationDeliveryStatus {
    @SerialName("queued") QUEUED,
    @SerialName("sent") SENT,
    @SerialName("failed") FAILED,
    @SerialName("read") READ,
}

@Serializable
data class AppNotification(
    val id: String,
    val recipientId: String,
    val channel: NotificationChannel,
    val title: String,
    val body: String,
    val status: NotificationDeliveryStatus,
    @Serializable(with = InstantSerializer::class)
    val createdAt: Instant,
    @Serializable(with = InstantSerializer::class)
    val sentAt: Instant? = null,
)
