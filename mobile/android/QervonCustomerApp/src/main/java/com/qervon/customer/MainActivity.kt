// =============================================================================
// File:           mobile/android/QervonCustomerApp/src/main/java/com/qervon/customer/MainActivity.kt
// Project:        Qervon Logistics Operating System (LOS)
// Author:         USDTG GROUP TECHNOLOGY LLC / Irfan Gedik
// Description:    Native Android Customer Application Activity with Jetpack Compose & Server-Side Isolated WebSocket
// =============================================================================

package com.qervon.customer

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import okhttp3.*
import org.json.JSONObject

class MainActivity : ComponentActivity() {

    private val assignedCourierId = "00000000-0000-0000-0000-000000000001"
    private val wsUrl = "ws://10.0.2.2:8080/ws/tracking/customer?courier_id=$assignedCourierId"

    private var courierLocationState = mutableStateOf("Atanan Kurye Konumu Bekleniyor...")
    private val client = OkHttpClient()

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate()
        connectProtectedWebSocket()

        setContent {
            CustomerScreen(courierLocationState.value)
        }
    }

    private fun connectProtectedWebSocket() {
        val request = Request.Builder().url(wsUrl).build()
        client.newWebSocket(request, object : WebSocketListener() {
            override fun onMessage(webSocket: WebSocket, text: String) {
                try {
                    val json = JSONObject(text)
                    val lat = json.getDouble("latitude")
                    val lon = json.getDouble("longitude")
                    courierLocationState.value = String.format("Atanan Kurye Konumu: Lat %.5f, Lon %.5f", lat, lon)
                } catch (e: Exception) {}
            }
        })
    }
}

@Composable
fun CustomerScreen(locationText: String) {
    Surface(
        modifier = Modifier.fillMaxSize(),
        color = Color(0xFF060913)
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(20.dp),
            verticalArrangement = Arrangement.SpaceBetween
        ) {
            // Header Bar
            Row(
                modifier = Modifier.fillMaxWidth(),
                horizontalArrangement = Arrangement.SpaceBetween,
                verticalAlignment = Alignment.CenterVertically
            ) {
                Column {
                    Text(
                        text = "QERVON MÜŞTERİ",
                        fontSize = 20.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color(0xFF10B981)
                    )
                    Text(
                        text = "Android Native • Sunucu Korumalı GPS",
                        fontSize = 11.sp,
                        color = Color.Gray
                    )
                }

                Box(
                    modifier = Modifier
                        .background(Color(0x2610B981), shape = RoundedCornerShape(8.dp))
                        .padding(horizontal = 8.dp, vertical = 4.dp)
                ) {
                    Text(
                        text = "SERVER ISOLATED 🛡️",
                        fontSize = 10.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color(0xFF10B981)
                    )
                }
            }

            // Assigned Courier Card
            Card(
                modifier = Modifier.fillMaxWidth(),
                shape = RoundedCornerShape(16.dp),
                colors = CardDefaults.cardColors(containerColor = Color(0xFF0F172A))
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        text = "SADECE ATANAN KURYE (SUNUCU KORUMALI)",
                        fontSize = 10.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color.Gray
                    )
                    Spacer(modifier = Modifier.height(4.dp))
                    Text(
                        text = "Ahmet Kurye (Motor 🏍️)",
                        fontSize = 16.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color(0xFF10B981)
                    )
                    Spacer(modifier = Modifier.height(6.dp))
                    Text(
                        text = locationText,
                        fontSize = 12.sp,
                        fontWeight = FontWeight.Bold,
                        color = Color(0xFF38BDF8)
                    )
                }
            }

            // Action Button
            Button(
                onClick = { },
                modifier = Modifier
                    .fillMaxWidth()
                    .height(54.dp),
                shape = RoundedCornerShape(16.dp),
                colors = ButtonDefaults.buttonColors(containerColor = Color(0xFF10B981))
            ) {
                Text(
                    text = "⚡ KURYEYİ ÇAĞIR (₺45.00)",
                    fontSize = 14.sp,
                    fontWeight = FontWeight.Bold,
                    color = Color.White
                )
            }
        }
    }
}
