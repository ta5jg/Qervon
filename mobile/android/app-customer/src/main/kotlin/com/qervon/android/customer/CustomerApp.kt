// =============================================================================
// File:           mobile/android/app-customer/src/main/kotlin/com/qervon/android/customer/CustomerApp.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   Top-level navigation: login/registration → optional biometric lock →
//   the three-tab main shell (New Order / My Orders / Account). Unlike
//   the courier app, self-registration is enabled here
//   (`POST /v1/auth/register` only ever provisions customers).
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.android.customer

import android.Manifest
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AccountCircle
import androidx.compose.material.icons.filled.AddCircle
import androidx.compose.material.icons.filled.List
import androidx.compose.material.icons.filled.SupportAgent
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import com.qervon.features.auth.BiometricLockScreen
import com.qervon.features.auth.LoginScreen
import com.qervon.features.auth.RegisterScreen
import com.qervon.features.customerorder.NewOrderScreen
import com.qervon.features.customerorder.OrderDetailScreen
import com.qervon.features.customerorder.OrderHistoryScreen
import com.qervon.features.customerprofile.CustomerProfileScreen
import com.qervon.features.customerprofile.CustomerSupportScreen

private object Routes {
    const val LOGIN = "login"
    const val REGISTER = "register"
    const val BIOMETRIC_LOCK = "biometric_lock"
    const val MAIN = "main"
    const val NEW_ORDER = "new_order"
    const val ORDERS = "orders"
    const val PROFILE = "profile"
    const val SUPPORT = "support"
    fun orderDetail(orderId: String) = "orderDetail/$orderId"
}

@Composable
fun CustomerApp(rootViewModel: CustomerRootViewModel = hiltViewModel()) {
    val rootState by rootViewModel.state.collectAsStateWithLifecycle()
    val navController = rememberNavController()

    val permissionLauncher = rememberLauncherForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) {}
    LaunchedEffect(Unit) {
        val permissions = mutableListOf(Manifest.permission.ACCESS_FINE_LOCATION)
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
                appTitle = "Qervon Müşteri",
                appSubtitle = "Hızlı ve güvenilir teslimat",
                showsRegistration = true,
                onAuthenticated = {
                    rootViewModel.onAuthenticated()
                    navController.navigate(Routes.MAIN) { popUpTo(Routes.LOGIN) { inclusive = true } }
                },
                onNavigateToRegister = { navController.navigate(Routes.REGISTER) },
            )
        }
        composable(Routes.REGISTER) {
            RegisterScreen(
                onRegistered = { navController.popBackStack() },
                onBack = { navController.popBackStack() },
            )
        }
        composable(Routes.BIOMETRIC_LOCK) {
            BiometricLockScreen(
                appTitle = "Qervon Müşteri",
                onUnlocked = {
                    rootViewModel.onBiometricUnlocked()
                    navController.navigate(Routes.MAIN) { popUpTo(Routes.BIOMETRIC_LOCK) { inclusive = true } }
                },
            )
        }
        composable(Routes.MAIN) {
            CustomerMainTabs(
                onOrderSelected = { orderId -> navController.navigate(Routes.orderDetail(orderId)) },
                onLoggedOut = {
                    rootViewModel.onLoggedOut()
                    navController.navigate(Routes.LOGIN) { popUpTo(Routes.MAIN) { inclusive = true } }
                },
            )
        }
        composable("orderDetail/{orderId}") {
            OrderDetailScreen(onBack = { navController.popBackStack() })
        }
    }
}

@Composable
private fun CustomerMainTabs(
    onOrderSelected: (orderId: String) -> Unit,
    onLoggedOut: () -> Unit,
) {
    val tabNavController = rememberNavController()

    Scaffold(
        bottomBar = {
            val currentEntry by tabNavController.currentBackStackEntryAsState()
            val currentRoute = currentEntry?.destination?.route
            NavigationBar {
                NavigationBarItem(
                    selected = currentRoute == Routes.NEW_ORDER,
                    onClick = { tabNavController.navigate(Routes.NEW_ORDER) { launchSingleTop = true } },
                    icon = { Icon(Icons.Filled.AddCircle, contentDescription = null) },
                    label = { Text("Yeni Sipariş") },
                )
                NavigationBarItem(
                    selected = currentRoute == Routes.ORDERS,
                    onClick = { tabNavController.navigate(Routes.ORDERS) { launchSingleTop = true } },
                    icon = { Icon(Icons.Filled.List, contentDescription = null) },
                    label = { Text("Siparişlerim") },
                )
                NavigationBarItem(
                    selected = currentRoute == Routes.PROFILE,
                    onClick = { tabNavController.navigate(Routes.PROFILE) { launchSingleTop = true } },
                    icon = { Icon(Icons.Filled.AccountCircle, contentDescription = null) },
                    label = { Text("Hesabım") },
                )
                NavigationBarItem(
                    selected = currentRoute == Routes.SUPPORT,
                    onClick = { tabNavController.navigate(Routes.SUPPORT) { launchSingleTop = true } },
                    icon = { Icon(Icons.Filled.SupportAgent, contentDescription = null) },
                    label = { Text("Destek") },
                )
            }
        },
    ) { padding ->
        NavHost(
            navController = tabNavController,
            startDestination = Routes.NEW_ORDER,
            modifier = Modifier.padding(padding),
        ) {
            composable(Routes.NEW_ORDER) { NewOrderScreen(onOrderCreated = onOrderSelected) }
            composable(Routes.ORDERS) { OrderHistoryScreen(onOrderSelected = onOrderSelected) }
            composable(Routes.PROFILE) { CustomerProfileScreen(onLoggedOut = onLoggedOut) }
            composable(Routes.SUPPORT) { CustomerSupportScreen() }
        }
    }
}
