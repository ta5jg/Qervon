// =============================================================================
// File:           mobile/android/app-courier/src/main/kotlin/com/qervon/android/courier/CourierApp.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Top-level navigation: auth (login only — the courier app has no
//   self-registration, couriers are provisioned by a tenant admin, see
//   `RegisterCourierRequest` in the backend) → optional biometric lock →
//   the four-tab main shell, with the proof-of-delivery flow as a
//   full-screen route reached from the Orders tab.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.android.courier

import android.Manifest
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AccountCircle
import androidx.compose.material.icons.filled.List
import androidx.compose.material.icons.filled.Wallet
import androidx.compose.material.icons.filled.WifiTethering
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.qervon.features.auth.BiometricLockScreen
import com.qervon.features.auth.LoginScreen
import com.qervon.features.dispatch.DispatchScreen
import com.qervon.features.earnings.EarningsScreen
import com.qervon.features.orders.OrdersScreen
import com.qervon.features.profile.CourierProfileScreen
import com.qervon.features.proof.ProofOfDeliveryScreen

private object Routes {
    const val LOGIN = "login"
    const val BIOMETRIC_LOCK = "biometric_lock"
    const val MAIN = "main"
    const val DISPATCH = "dispatch"
    const val ORDERS = "orders"
    const val EARNINGS = "earnings"
    const val PROFILE = "profile"
    fun proof(orderId: String, isCash: Boolean) = "proof/$orderId/$isCash"
}

@Composable
fun CourierApp(rootViewModel: CourierRootViewModel = hiltViewModel()) {
    val rootState by rootViewModel.state.collectAsStateWithLifecycle()
    val navController = rememberNavController()
    val context = LocalContext.current

    val permissionLauncher = rememberLauncherForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) {}
    LaunchedEffect(Unit) {
        val permissions = mutableListOf(Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.CAMERA)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            permissions += Manifest.permission.POST_NOTIFICATIONS
        }
        permissionLauncher.launch(permissions.toTypedArray())
    }

    val startDestination = when {
        !rootState.hasSession -> Routes.LOGIN
        !rootState.isUnlocked -> Routes.BIOMETRIC_LOCK
        else -> Routes.MAIN
    }

    NavHost(navController = navController, startDestination = startDestination) {
        composable(Routes.LOGIN) {
            LoginScreen(
                appTitle = "Qervon Kurye",
                appSubtitle = "Teslimatlarınızı yönetin",
                showsRegistration = false,
                onAuthenticated = {
                    rootViewModel.onAuthenticated()
                    navController.navigate(Routes.MAIN) { popUpTo(Routes.LOGIN) { inclusive = true } }
                },
                onNavigateToRegister = {},
            )
        }
        composable(Routes.BIOMETRIC_LOCK) {
            BiometricLockScreen(
                appTitle = "Qervon Kurye",
                onUnlocked = {
                    rootViewModel.onBiometricUnlocked()
                    navController.navigate(Routes.MAIN) { popUpTo(Routes.BIOMETRIC_LOCK) { inclusive = true } }
                },
            )
        }
        composable(Routes.MAIN) {
            CourierMainTabs(
                onStartDelivery = { orderId, isCash -> navController.navigate(Routes.proof(orderId, isCash)) },
                onLoggedOut = {
                    rootViewModel.onLoggedOut()
                    navController.navigate(Routes.LOGIN) { popUpTo(Routes.MAIN) { inclusive = true } }
                },
            )
        }
        composable("proof/{orderId}/{isCash}") { backStackEntry ->
            val orderId = backStackEntry.arguments?.getString("orderId").orEmpty()
            val isCash = backStackEntry.arguments?.getString("isCash")?.toBoolean() ?: false
            ProofOfDeliveryScreen(
                orderId = orderId,
                isCashOrder = isCash,
                onDelivered = { navController.popBackStack() },
                onClose = { navController.popBackStack() },
            )
        }
    }
}

@Composable
private fun CourierMainTabs(
    onStartDelivery: (orderId: String, isCash: Boolean) -> Unit,
    onLoggedOut: () -> Unit,
) {
    val tabNavController = rememberNavController()

    Scaffold(
        bottomBar = {
            val currentEntry by tabNavController.currentBackStackEntryAsState()
            val currentRoute = currentEntry?.destination?.route
            NavigationBar {
                NavigationBarItem(
                    selected = currentRoute == Routes.DISPATCH,
                    onClick = { tabNavController.navigate(Routes.DISPATCH) { launchSingleTop = true } },
                    icon = { Icon(Icons.Filled.WifiTethering, contentDescription = null) },
                    label = { Text("Panel") },
                )
                NavigationBarItem(
                    selected = currentRoute == Routes.ORDERS,
                    onClick = { tabNavController.navigate(Routes.ORDERS) { launchSingleTop = true } },
                    icon = { Icon(Icons.Filled.List, contentDescription = null) },
                    label = { Text("İşlerim") },
                )
                NavigationBarItem(
                    selected = currentRoute == Routes.EARNINGS,
                    onClick = { tabNavController.navigate(Routes.EARNINGS) { launchSingleTop = true } },
                    icon = { Icon(Icons.Filled.Wallet, contentDescription = null) },
                    label = { Text("Kazanç") },
                )
                NavigationBarItem(
                    selected = currentRoute == Routes.PROFILE,
                    onClick = { tabNavController.navigate(Routes.PROFILE) { launchSingleTop = true } },
                    icon = { Icon(Icons.Filled.AccountCircle, contentDescription = null) },
                    label = { Text("Hesap") },
                )
            }
        },
    ) { padding ->
        NavHost(
            navController = tabNavController,
            startDestination = Routes.DISPATCH,
            modifier = Modifier.padding(padding),
        ) {
            composable(Routes.DISPATCH) { DispatchScreen() }
            composable(Routes.ORDERS) {
                OrdersScreen(onStartDelivery = onStartDelivery)
            }
            composable(Routes.EARNINGS) { EarningsScreen() }
            composable(Routes.PROFILE) { CourierProfileScreen(onLoggedOut = onLoggedOut) }
        }
    }
}
