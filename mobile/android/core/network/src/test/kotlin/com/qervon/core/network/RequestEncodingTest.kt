// =============================================================================
// File:           mobile/android/core/network/src/test/kotlin/com/qervon/core/network/RequestEncodingTest.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.network

import com.qervon.core.common.JsonConfig
import org.junit.Assert.assertTrue
import org.junit.Test

class RequestEncodingTest {

    @Test
    fun `login body encodes snake_case field names`() {
        val json = JsonConfig.shared.encodeToString(
            LoginBody.serializer(),
            LoginBody(email = "a@b.com", password = "secret", tenantSlug = "acme"),
        )
        assertTrue(json.contains("\"tenant_slug\":\"acme\""))
        assertTrue(json.contains("\"email\":\"a@b.com\""))
    }

    @Test
    fun `complete delivery body omits nulls but includes required flags`() {
        val json = JsonConfig.shared.encodeToString(
            CompleteDeliveryBody.serializer(),
            CompleteDeliveryBody(recipientName = "Ali", qrBarcodeVerified = true),
        )
        assertTrue(json.contains("\"qr_barcode_verified\":true"))
        assertTrue(json.contains("\"recipient_name\":\"Ali\""))
    }

    @Test
    fun `complete pickup body includes evidence url`() {
        val json = JsonConfig.shared.encodeToString(
            CompletePickupBody.serializer(),
            CompletePickupBody("/v1/uploads/delivery-photos/order/photo.jpg"),
        )
        assertTrue(json.contains("\"pickup_photo_evidence_url\":\"/v1/uploads/delivery-photos/order/photo.jpg\""))
    }
}
