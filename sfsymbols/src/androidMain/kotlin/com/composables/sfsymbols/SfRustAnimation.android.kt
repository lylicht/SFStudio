package com.composables.sfsymbols

import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color
import com.composables.sfsymbols.core.SfGlyph
import com.composables.sfsymbols.core.SfScaleId

internal actual fun sfRustGlyphData(name: String, scale: SfScaleId): SfGlyph? =
    RustFfi.glyphData(name, scale)

internal actual fun sfRustGlyphPoints(symbol: SfSymbol, weight: Float, scale: SfScale): FloatArray? =
    RustFfi.points(symbol.name, scale, weight)

internal actual fun sfRustGlyphBounds(symbol: SfSymbol, weight: Float, scale: SfScale): FloatArray? =
    RustFfi.bounds(symbol.name, scale, weight)

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
): FloatArray? = RustFfi.frames(symbol, effect, time, rtl, variableValue, mode, tint, palette, gradient)

internal actual fun sfRustDrawFieldLevels(
    mask: ByteArray,
    originX: Float,
    originY: Float,
    cell: Float,
    symbolBounds: Rect,
    ties: FloatArray,
): ByteArray? = RustFfi.drawFieldLevels(mask, originX, originY, cell, symbolBounds, ties)

private object RustFfi {
    private val available by lazy { runCatching { System.loadLibrary("sfstudio_ffi") }.isSuccess }

    external fun glyphPoints(name: String, scale: Int, weight: Float): FloatArray?
    external fun glyphBounds(name: String, scale: Int, weight: Float): FloatArray?
    external fun glyphJson(name: String, scale: Int): ByteArray?
    external fun animationJson(input: ByteArray): ByteArray?
    external fun drawFieldJson(input: ByteArray): ByteArray?

    fun points(name: String, scale: SfScale, weight: Float): FloatArray? {
        if (!available) return null
        val scaleId = when (scale) {
            SfScale.Medium -> 0
            SfScale.Small -> 1
            SfScale.Large -> 2
        }
        return try {
            glyphPoints(name, scaleId, weight)?.takeIf { points -> points.all { it.isFinite() } }
        } catch (_: Exception) {
            null
        } catch (_: LinkageError) {
            null
        }
    }

    fun bounds(name: String, scale: SfScale, weight: Float): FloatArray? {
        if (!available) return null
        val scaleId = when (scale) {
            SfScale.Medium -> 0
            SfScale.Small -> 1
            SfScale.Large -> 2
        }
        return try {
            glyphBounds(name, scaleId, weight)?.takeIf {
                it.size == 4 && it.all(Float::isFinite) && it[0] <= it[2] && it[1] <= it[3]
            }
        } catch (_: Exception) {
            null
        } catch (_: LinkageError) {
            null
        }
    }

    fun glyphData(name: String, scale: SfScaleId): SfGlyph? {
        if (!available) return null
        val scaleId = when (scale) {
            SfScaleId.Medium -> 0
            SfScaleId.Small -> 1
            SfScaleId.Large -> 2
        }
        return try {
            glyphJson(name, scaleId)?.decodeToString()?.let(::parseRustGlyphData)
        } catch (_: Exception) {
            null
        } catch (_: LinkageError) {
            null
        }
    }

    fun frames(
        symbol: SfSymbol,
        effect: SfSymbolEffect,
        time: Float,
        rtl: Boolean,
        variableValue: Float?,
        mode: SfRenderingMode,
        tint: Color,
        palette: List<Color>,
        gradient: Boolean,
    ): FloatArray? = try {
        if (!available) return null
        val request = buildRustAnimationRequest(symbol, effect, time, rtl, variableValue, mode, tint, palette, gradient)
        val response = animationJson(request) ?: return null
        parseRustAnimationFrames(response.decodeToString(), symbol.layers.size) ?: return null
    } catch (_: Exception) {
        null
    } catch (_: LinkageError) {
        null
    }

    fun drawFieldLevels(
        mask: ByteArray,
        originX: Float,
        originY: Float,
        cell: Float,
        symbolBounds: Rect,
        ties: FloatArray,
    ): ByteArray? = try {
        if (!available) return null
        val request = buildRustDrawFieldRequest(mask, originX, originY, cell, symbolBounds, ties)
        val response = drawFieldJson(request) ?: return null
        parseRustDrawFieldLevels(response.decodeToString(), mask.size) ?: return null
    } catch (_: Exception) {
        null
    } catch (_: LinkageError) {
        null
    }
}
