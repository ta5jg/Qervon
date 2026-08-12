// =============================================================================
// File:           mobile/android/core/location/src/main/kotlin/com/qervon/core/location/CourierLocationService.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Foreground `Service` that requests FusedLocationProvider updates every
//   ~8 seconds while the courier is online and reports each sample via
//   [LocationReporter.reporter]. Started/stopped by the dispatch feature
//   when the courier toggles online/offline — mirrors the iOS client's
//   `CourierLocationBroadcaster`, but as an explicit Android foreground
//   service (required for reliable updates while the screen is off).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.core.location

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.BatteryManager
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import com.google.android.gms.location.FusedLocationProviderClient
import com.google.android.gms.location.LocationCallback
import com.google.android.gms.location.LocationRequest
import com.google.android.gms.location.LocationResult
import com.google.android.gms.location.LocationServices
import com.google.android.gms.location.Priority
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.launch

private const val NOTIFICATION_CHANNEL_ID = "qervon_courier_location"
private const val NOTIFICATION_ID = 4200
private const val UPDATE_INTERVAL_MS = 8_000L

class CourierLocationService : Service() {

    companion object {
        /** Set once by the app's composition root before the first
         * [start] call — never null in practice for a correctly wired app,
         * but the Service degrades to a no-op reporter rather than crash
         * if it somehow is. */
        @Volatile
        var reporter: LocationReporter? = null

        fun start(context: Context) {
            val intent = Intent(context, CourierLocationService::class.java)
            context.startForegroundService(intent)
        }

        fun stop(context: Context) {
            context.stopService(Intent(context, CourierLocationService::class.java))
        }
    }

    private val serviceScope = CoroutineScope(SupervisorJob() + Dispatchers.IO)
    private val fusedClient: FusedLocationProviderClient by lazy {
        LocationServices.getFusedLocationProviderClient(this)
    }

    private val locationCallback = object : LocationCallback() {
        override fun onLocationResult(result: LocationResult) {
            val location = result.lastLocation ?: return
            val speedKmh = if (location.hasSpeed()) (location.speed * 3.6) else null
            serviceScope.launch {
                reporter?.reportLocation(location.latitude, location.longitude, speedKmh, batteryPercent())
            }
        }
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        startForeground(NOTIFICATION_ID, buildNotification())
        requestLocationUpdates()
        return START_STICKY
    }

    override fun onDestroy() {
        fusedClient.removeLocationUpdates(locationCallback)
        serviceScope.cancel()
        super.onDestroy()
    }

    override fun onBind(intent: Intent?): IBinder? = null

    @Suppress("MissingPermission") // Caller is required to have requested ACCESS_FINE_LOCATION before starting.
    private fun requestLocationUpdates() {
        val request = LocationRequest.Builder(Priority.PRIORITY_HIGH_ACCURACY, UPDATE_INTERVAL_MS)
            .setMinUpdateIntervalMillis(UPDATE_INTERVAL_MS / 2)
            .build()
        fusedClient.requestLocationUpdates(request, locationCallback, mainLooper)
    }

    private fun batteryPercent(): Int? {
        val batteryManager = getSystemService(BATTERY_SERVICE) as? BatteryManager ?: return null
        val level = batteryManager.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY)
        return if (level in 0..100) level else null
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
        val manager = getSystemService(NotificationManager::class.java)
        val channel = NotificationChannel(
            NOTIFICATION_CHANNEL_ID,
            "Kurye Konum Paylaşımı",
            NotificationManager.IMPORTANCE_LOW,
        ).apply { description = "Çevrimiçiyken konumunuz sipariş takibi için paylaşılır." }
        manager.createNotificationChannel(channel)
    }

    private fun buildNotification(): Notification {
        val launchIntent = packageManager.getLaunchIntentForPackage(packageName)
        val pendingIntent = launchIntent?.let {
            PendingIntent.getActivity(this, 0, it, PendingIntent.FLAG_IMMUTABLE)
        }
        return NotificationCompat.Builder(this, NOTIFICATION_CHANNEL_ID)
            .setContentTitle("Qervon Kurye — Çevrimiçi")
            .setContentText("Konumunuz aktif siparişler için paylaşılıyor.")
            .setSmallIcon(android.R.drawable.ic_menu_mylocation)
            .setOngoing(true)
            .setContentIntent(pendingIntent)
            .build()
    }
}
