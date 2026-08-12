// =============================================================================
// File:           mobile/android/feature/proof/src/main/kotlin/com/qervon/features/proof/SignatureCaptureScreen.kt
// Project:        Qervon
// Author:         USDTG GROUP TECHNOLOGY LLC
// Developer:      Irfan Gedik
// Created Date:   2026-08-12
// Version:        0.1.0
//
// Description:
//   A finger-drawn signature pad rendered on a Compose `Canvas`, exported
//   as a base64-encoded PNG for `digital_signature_base64` — the Android
//   equivalent of the iOS client's `PKCanvasView`-based signature screen.
//
// License:
//   Qervon License v1.0 — see LICENSE in the repository root.
// =============================================================================

package com.qervon.features.proof

import android.graphics.Bitmap
import android.graphics.Canvas as AndroidCanvas
import android.graphics.Paint
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.gestures.detectDragGestures
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.aspectRatio
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.OutlinedButton
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.StrokeJoin
import androidx.compose.ui.graphics.asAndroidPath
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.unit.dp
import com.qervon.core.designsystem.QervonPrimaryButton
import com.qervon.core.designsystem.QervonSpacing
import java.io.ByteArrayOutputStream
import android.util.Base64

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SignatureCaptureScreen(onCaptured: (base64Png: String) -> Unit, onClose: () -> Unit) {
    var paths by remember { mutableStateOf(listOf<Path>()) }
    var currentPath by remember { mutableStateOf(Path()) }
    var canvasSize by remember { mutableStateOf(androidx.compose.ui.geometry.Size.Zero) }

    Scaffold(topBar = { TopAppBar(title = { Text("İmza Alın") }) }) { padding ->
        Column(
            modifier = Modifier.fillMaxSize().padding(padding).padding(QervonSpacing.md),
            verticalArrangement = Arrangement.spacedBy(QervonSpacing.md),
        ) {
            Text("Alıcının imzasını aşağıdaki alana çizmesini isteyin.")
            Canvas(
                modifier = Modifier
                    .fillMaxWidth()
                    .aspectRatio(1.6f)
                    .background(Color.White)
                    .border(1.dp, Color.LightGray)
                    .pointerInput(Unit) {
                        detectDragGestures(
                            onDragStart = { offset ->
                                currentPath = Path().apply { moveTo(offset.x, offset.y) }
                            },
                            onDrag = { change, _ ->
                                currentPath = Path().apply {
                                    addPath(currentPath)
                                    lineTo(change.position.x, change.position.y)
                                }
                            },
                            onDragEnd = {
                                paths = paths + currentPath
                                currentPath = Path()
                            },
                        )
                    },
            ) {
                canvasSize = size
                (paths + currentPath).forEach { path ->
                    drawPath(
                        path = path,
                        color = Color.Black,
                        style = Stroke(width = 4f, cap = StrokeCap.Round, join = StrokeJoin.Round),
                    )
                }
            }
            Row(horizontalArrangement = Arrangement.spacedBy(QervonSpacing.sm)) {
                OutlinedButton(onClick = { paths = emptyList(); currentPath = Path() }) { Text("Temizle") }
                OutlinedButton(onClick = onClose) { Text("Vazgeç") }
            }
            QervonPrimaryButton(
                text = "İmzayı Onayla",
                enabled = paths.isNotEmpty(),
                onClick = {
                    onCaptured(exportSignatureAsBase64Png(paths, canvasSize))
                },
            )
        }
    }
}

private fun exportSignatureAsBase64Png(paths: List<Path>, size: androidx.compose.ui.geometry.Size): String {
    val width = size.width.toInt().coerceAtLeast(1)
    val height = size.height.toInt().coerceAtLeast(1)
    val bitmap = Bitmap.createBitmap(width, height, Bitmap.Config.ARGB_8888)
    val androidCanvas = AndroidCanvas(bitmap)
    androidCanvas.drawColor(android.graphics.Color.WHITE)
    val paint = Paint().apply {
        color = android.graphics.Color.BLACK
        style = Paint.Style.STROKE
        strokeWidth = 4f
        strokeCap = Paint.Cap.ROUND
        strokeJoin = Paint.Join.ROUND
        isAntiAlias = true
    }
    paths.forEach { path -> androidCanvas.drawPath(path.asAndroidPath(), paint) }

    val stream = ByteArrayOutputStream()
    bitmap.compress(Bitmap.CompressFormat.PNG, 100, stream)
    return Base64.encodeToString(stream.toByteArray(), Base64.NO_WRAP)
}
