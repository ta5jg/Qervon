// =============================================================================
// File:           mobile/android/feature/proof/src/main/kotlin/com/qervon/features/proof/PhotoCaptureScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   CameraX photo capture saved to the app's private local storage. After
//   capture the courier reviews the photo and taps Continue; the file is
//   uploaded later by the pickup/delivery view model.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.proof

import android.graphics.BitmapFactory
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.camera.core.CameraSelector
import androidx.camera.core.ImageCapture
import androidx.camera.core.ImageCaptureException
import androidx.camera.core.Preview
import androidx.camera.lifecycle.ProcessCameraProvider
import androidx.camera.view.PreviewView
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Camera
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.platform.LocalContext
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.core.content.ContextCompat
import com.qervon.core.designsystem.QervonColors
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing
import java.io.File
import java.text.SimpleDateFormat
import java.util.Locale
import java.util.concurrent.Executors

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun PhotoCaptureScreen(
    title: String = "Teslimat Fotoğrafı",
    onCaptured: (localPath: String) -> Unit,
    onClose: () -> Unit,
) {
    val context = LocalContext.current
    val lifecycleOwner = LocalLifecycleOwner.current
    val cameraExecutor = remember { Executors.newSingleThreadExecutor() }
    val imageCapture = remember { ImageCapture.Builder().build() }
    var capturedPath by remember { mutableStateOf<String?>(null) }
    var errorMessage by remember { mutableStateOf<String?>(null) }
    var capturing by remember { mutableStateOf(false) }

    val galleryLauncher = rememberLauncherForActivityResult(ActivityResultContracts.GetContent()) { uri: Uri? ->
        if (uri == null) return@rememberLauncherForActivityResult
        runCatching {
            val outputDir = File(context.filesDir, "delivery_photos").apply { mkdirs() }
            val outputFile = File(outputDir, "gallery_${System.currentTimeMillis()}.jpg")
            context.contentResolver.openInputStream(uri)?.use { input ->
                outputFile.outputStream().use { output -> input.copyTo(output) }
            }
            outputFile.absolutePath
        }.onSuccess { path ->
            capturedPath = path
            errorMessage = null
        }.onFailure {
            errorMessage = "Galeriden fotoğraf alınamadı."
        }
    }

    DisposableEffect(Unit) {
        onDispose { cameraExecutor.shutdown() }
    }

    val previewPath = capturedPath
    if (previewPath != null) {
        val bitmap = remember(previewPath) { BitmapFactory.decodeFile(previewPath) }
        Scaffold(topBar = { TopAppBar(title = { Text(title) }) }) { padding ->
            Column(
                modifier = Modifier.fillMaxSize().padding(padding).padding(QervonSpacing.md),
                verticalArrangement = Arrangement.spacedBy(QervonSpacing.md),
            ) {
                if (bitmap != null) {
                    Image(
                        bitmap = bitmap.asImageBitmap(),
                        contentDescription = "Çekilen fotoğraf",
                        modifier = Modifier.fillMaxWidth().weight(1f),
                    )
                } else {
                    Text("Fotoğraf kaydedildi. Devam ederek sunucuya yükleyin.")
                }
                QervonPrimaryButton(text = "Devam et", onClick = { onCaptured(previewPath) })
                OutlinedButton(onClick = { capturedPath = null }, modifier = Modifier.fillMaxWidth()) {
                    Text("Yeniden çek")
                }
            }
        }
        return
    }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text(title) },
                navigationIcon = {
                    IconButton(onClick = onClose) { Icon(Icons.Filled.Close, contentDescription = "Kapat") }
                },
            )
        },
        floatingActionButton = {
            FloatingActionButton(onClick = {
                if (capturing) return@FloatingActionButton
                capturing = true
                capturePhoto(context, imageCapture, cameraExecutor, ContextCompat.getMainExecutor(context), { path ->
                    capturing = false
                    capturedPath = path
                    errorMessage = null
                }, { message ->
                    capturing = false
                    errorMessage = message
                })
            }) {
                Icon(Icons.Filled.Camera, contentDescription = "Fotoğraf çek")
            }
        },
        floatingActionButtonPosition = androidx.compose.material3.FabPosition.Center,
    ) { padding ->
        Box(modifier = Modifier.fillMaxSize().padding(padding)) {
            AndroidView(
                modifier = Modifier.fillMaxSize(),
                factory = { ctx ->
                    val previewView = PreviewView(ctx)
                    ProcessCameraProvider.getInstance(ctx).also { future ->
                        future.addListener(
                            {
                                try {
                                    val provider = future.get()
                                    val preview = Preview.Builder().build().also {
                                        it.setSurfaceProvider(previewView.surfaceProvider)
                                    }
                                    provider.unbindAll()
                                    provider.bindToLifecycle(
                                        lifecycleOwner,
                                        CameraSelector.DEFAULT_BACK_CAMERA,
                                        preview,
                                        imageCapture,
                                    )
                                } catch (_: Exception) {
                                    errorMessage = "Kamera açılamadı. Galeriden bir fotoğraf seçebilirsiniz."
                                }
                            },
                            ContextCompat.getMainExecutor(ctx),
                        )
                    }
                    previewView
                },
            )
            Column(
                modifier = Modifier.align(Alignment.BottomCenter).padding(16.dp).padding(bottom = 72.dp),
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                errorMessage?.let { Text(it, color = QervonColors.Danger) }
                OutlinedButton(onClick = { galleryLauncher.launch("image/*") }) {
                    Text("Galeriden seç")
                }
            }
        }
    }
}

private fun capturePhoto(
    context: android.content.Context,
    imageCapture: ImageCapture,
    executor: java.util.concurrent.ExecutorService,
    mainExecutor: java.util.concurrent.Executor,
    onSaved: (String) -> Unit,
    onFailed: (String) -> Unit,
) {
    val timestamp = SimpleDateFormat("yyyyMMdd_HHmmss", Locale.US).format(java.util.Date())
    val outputDir = File(context.filesDir, "delivery_photos").apply { mkdirs() }
    val outputFile = File(outputDir, "proof_$timestamp.jpg")
    val outputOptions = ImageCapture.OutputFileOptions.Builder(outputFile).build()

    imageCapture.takePicture(
        outputOptions,
        executor,
        object : ImageCapture.OnImageSavedCallback {
            override fun onImageSaved(output: ImageCapture.OutputFileResults) {
                mainExecutor.execute { onSaved(outputFile.absolutePath) }
            }

            override fun onError(exception: ImageCaptureException) {
                mainExecutor.execute {
                    onFailed("Fotoğraf çekilemedi. Kamerayı yeniden deneyin veya galeriden seçin.")
                }
            }
        },
    )
}
