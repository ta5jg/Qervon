// =============================================================================
// File:           mobile/android/core/common/src/test/kotlin/com/qervon/core/common/JsonCodingTest.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.common

import com.qervon.core.common.model.Order
import com.qervon.core.common.model.OrderStatus
import com.qervon.core.common.model.PaymentMethod
import com.qervon.core.common.model.decodeAccessTokenClaims
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test
import java.time.Instant
import java.util.Base64

class JsonCodingTest {

    @Test
    fun `decodes OrderResponse shape from backend`() {
        val json = """
            {
                "id": "019ff5cd-f08b-73c2-8f77-07c852fbdaa1",
                "customer_id": "019ff5cd-f08b-73c2-8f77-07c852fbdaa2",
                "pickup": { "latitude": 41.0, "longitude": 29.0, "label": "Alım" },
                "dropoff": { "latitude": 41.1, "longitude": 29.1, "label": "Teslim" },
                "status": "courier_assigned",
                "fare": { "amount_minor": 4500, "currency": "TRY" },
                "assigned_courier_id": "019ff5cd-f08b-73c2-8f77-07c852fbdaa3",
                "created_at": "2026-08-12T10:00:00Z",
                "delivered_at": null,
                "returned_at": null,
                "payment_method": "cash",
                "payment_collected": false
            }
        """.trimIndent()

        val order = JsonConfig.shared.decodeFromString(Order.serializer(), json)
        assertEquals(OrderStatus.COURIER_ASSIGNED, order.status)
        assertEquals(4500L, order.fare.amountMinor)
        assertEquals(PaymentMethod.CASH, order.paymentMethod)
        assertNull(order.deliveredAt)
    }

    @Test
    fun `decodes fractional seconds timestamp`() {
        val instant = Instant.parse("2026-08-12T10:00:00.123456Z")
        assertTrue(instant.epochSecond > 0)
    }

    @Test
    fun `decodes access token claims without a signing secret`() {
        val payload = """{"subject":"019ff5cd-f08b-73c2-8f77-07c852fbdaa1","tenant_id":"019ff5cd-f08b-73c2-8f77-07c852fbdaa2","role":"courier","expires_at":9999999999}"""
        val encoded = Base64.getUrlEncoder().withoutPadding().encodeToString(payload.toByteArray())
        val token = "qv1.$encoded.fakesignature"

        val claims = decodeAccessTokenClaims(token)
        assertEquals(com.qervon.core.common.model.UserRole.COURIER, claims.role)
        assertTrue(!claims.isExpired)
    }
}
