package com.composables.sfsymbols

import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.add
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

internal actual fun sfRustGlyphPoints(symbol: SfSymbol, weight: Float, scale: SfScale): FloatArray? =
    RustFfi.points(symbol.name, scale, weight)

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
        val request = buildJsonObject {
            put("mask", buildJsonArray { mask.forEach { add(it.toInt() and 0xff) } })
            put("origin", buildJsonArray { add(originX); add(originY) })
            put("cell", cell)
            put("symbolBounds", buildJsonArray {
                add(symbolBounds.left); add(symbolBounds.top); add(symbolBounds.right); add(symbolBounds.bottom)
            })
            put("ties", buildJsonArray { ties.forEach { add(it) } })
        }.toString().toByteArray(Charsets.UTF_8)
        val response = drawFieldJson(request) ?: return null
        val values = Json.parseToJsonElement(response.decodeToString()).jsonObject["levels"]
            ?.takeUnless { it == JsonNull }?.jsonArray ?: return null
        if (values.size != mask.size) return null
        ByteArray(values.size) { index ->
            val value = values[index].jsonPrimitive.content.toIntOrNull()?.takeIf { it in 0..255 } ?: return null
            value.toByte()
        }
    } catch (_: Exception) {
        null
    } catch (_: LinkageError) {
        null
    }
}
