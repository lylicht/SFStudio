@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package com.composables.sfsymbols

import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color
import com.composables.sfsymbols.ffi.sfstudio_glyph_points
import kotlinx.cinterop.addressOf
import kotlinx.cinterop.convert
import kotlinx.cinterop.reinterpret
import kotlinx.cinterop.usePinned

internal actual fun sfRustGlyphPoints(symbol: SfSymbol, weight: Float, scale: SfScale): FloatArray? {
    val name = symbol.name.encodeToByteArray()
    if (name.isEmpty()) return null
    val scaleId = when (scale) {
        SfScale.Medium -> 0
        SfScale.Small -> 1
        SfScale.Large -> 2
    }
    return try {
        name.usePinned { pinnedName ->
            val namePointer = pinnedName.addressOf(0).reinterpret()
            val required = sfstudio_glyph_points(
                namePointer, name.size.convert(), scaleId, weight, null, 0uL,
            )
            if (required < 0L || required > Int.MAX_VALUE.toLong()) return null
            val count = required.toInt()
            if (count == 0) return FloatArray(0)
            val points = FloatArray(count)
            points.usePinned { pinnedPoints ->
                val written = sfstudio_glyph_points(
                    namePointer, name.size.convert(), scaleId, weight,
                    pinnedPoints.addressOf(0), count.convert(),
                )
                if (written != required) return null
            }
            points.takeIf { result -> result.all { it.isFinite() } }
        }
    } catch (_: Exception) {
        null
    }
}

internal actual fun sfRustAnimationFrames(
    symbol: SfSymbol,
    effect: SfSymbolEffect,
    time: Float,
    rtl: Boolean,
    variableValue: Float?,
    mode: SfRenderingMode,
    tint: Color,
    palette: List<Color>,
    gradient: Boolean,
): FloatArray? = null

internal actual fun sfRustDrawFieldLevels(
    mask: ByteArray,
    originX: Float,
    originY: Float,
    cell: Float,
    symbolBounds: Rect,
    ties: FloatArray,
): ByteArray? = null
