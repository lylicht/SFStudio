package com.composables.sfsymbols

import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color

internal actual fun sfRustGlyphPoints(symbol: SfSymbol, weight: Float, scale: SfScale): FloatArray? =
    RustGlyphPoints.points(symbol.name, scale, weight)

private object RustGlyphPoints {
    private val available by lazy {
        runCatching { System.loadLibrary("sfstudio_ffi") }.isSuccess
    }

    fun points(name: String, scale: SfScale, weight: Float): FloatArray? =
        if (available) runCatching { glyphPoints(name, scale.nativeId, weight) }.getOrNull() else null

    external fun glyphPoints(name: String, scale: Int, weight: Float): FloatArray?

    private val SfScale.nativeId: Int
        get() = when (this) {
            SfScale.Medium -> 0
            SfScale.Small -> 1
            SfScale.Large -> 2
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
