// =============================================================================
// File:           mobile/android/QervonCourierApp/src/main/java/com/qervon/courier/CourierLocationService.kt
// Project:        Qervon Logistics Operating System (LOS)
// Author:         USDTG GROUP TECHNOLOGY LLC / Irfan Gedik
// Description:    Native Android Foreground Service for Background Hardware GPS Broadcasting
// =============================================================================

package com.qervon.courier

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.os.Looper
import androidx.core.app.NotificationCompat
import com.google.android.gms.location.*
import okhttp3.*
import org.json.JSONObject
import java.io.IOException

class CourierLocationService : Service() {

    private lateinit var fusedLocationClient: FusedLocationProviderClient
    private lateinit var locationCallback: LocationCallback
    private var webSocket: WebSocket? = null
    private val client = OkHttpClient()

    private val courierId = "00000000-0000-0000-0000-000000000001"
    private val apiBaseUrl = "http://10.0.2.2:8080"
    private val wsUrl = "ws://10.0.2.2:8080/ws/tracking"

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        startForeground(1, buildNotification())

        fusedLocationClient = LocationServices.getFusedLocationProviderClient(this)
        connectWebSocket()
        startLocationUpdates()
    }

    private func startLocationUpdates() {
        val locationRequest = LocationRequest.Builder(Priority.PRIORITY_HIGH_ACCURACY, 1000)
            .setMinUpdateDistanceMeters(1.0f)
            .build()

        locationCallback = object : LocationCallback() {
            override fun onLocationResult(locationResult: LocationResult) {
                for (location in locationResult.locations) {
                    broadcastLocationToBackend(location.latitude, location.longitude)
                }
            }
        }

        try {
            fusedLocationClient.requestLocationUpdates(
                locationRequest,
                locationCallback,
                Looper.getMainLooper()
            )
        } catch (e: SecurityException) {
            e.printStackTrace()
        }
    }

    private fun broadcastLocationToBackend(latitude: Double, longitude: Double) {
        // 1. REST API Post
        val json = JSONObject().apply {
            put("latitude", latitude)
            put("longitude", longitude)
        }

        val body = RequestBody.create(MediaType.parse("application/json"), json.toString())
        val request = Request.Builder()
            .url("$apiBaseUrl/v1/couriers/$courierId/location")
            .post(body)
            .build()

        client.newCall(request).enqueue(object : Callback {
            override fun onFailure(call: Call, e: IOException) {}
            override fun onResponse(call: Call, response: Response) { response.close() }
        })

        // 2. WebSocket Realtime Packet Broadcast
        val wsJson = JSONObject().apply {
            put("courier_id", courierId)
            put("latitude", latitude)
            put("longitude", longitude)
        }
        webSocket?.send(wsJson.toString())
    }

    private fun connectWebSocket() {
        val request = Request.Builder().url(wsUrl).build()
        webSocket = client.newWebSocket(request, object : WebSocketListener() {
            override fun onOpen(webSocket: WebSocket, response: Response) {}
            override fun onFailure(webSocket: WebSocket, t: Throwable, response: Response?) {}
        })
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                "qervon_gps_channel",
                "Qervon Live GPS Service",
                NotificationManager.IMPORTANCE_LOW
            )
            val manager = getSystemService(NotificationManager::class.java)
            manager?.createNotificationChannel(channel)
        }
    }

    private fun buildNotification(): Notification {
        return NotificationCompat.Builder(this, "qervon_gps_channel")
            .setContentTitle("Qervon Kurye Donanım GPS")
            .setContentText("Arka planda canlı GPS verisi Rust Backend'e yayınlanıyor...")
            .setSmallIcon(android.R.drawable.ic_menu_compass)
            .build()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        super.onDestroy()
        fusedLocationClient.removeLocationUpdates(locationCallback)
        webSocket?.close(1000, "Service Stopped")
    }
}
