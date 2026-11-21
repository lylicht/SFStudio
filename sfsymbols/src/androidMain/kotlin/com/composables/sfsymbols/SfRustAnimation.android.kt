package com.composables.sfsymbols

import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.colorspace.ColorSpaces
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
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
        val includeAppearance = tint.isRustRepresentable() && palette.all { it.isRustRepresentable() }
        val request = buildJsonObject {
            put("symbol", symbol.name)
            put("time", time)
            put("rtl", rtl)
            put("effect", effect.toRustJson())
            if (variableValue != null) put("variableValue", variableValue)
            if (includeAppearance) put("rendering", buildJsonObject {
                put("mode", mode.toRustString())
                put("tint", tint.toRustRgba())
                put("palette", buildJsonArray { palette.forEach { add(it.toRustRgba()) } })
                put("gradient", gradient)
            })
        }.toString().toByteArray(Charsets.UTF_8)
        val response = animationJson(request) ?: return null
        val layers = Json.parseToJsonElement(response.decodeToString()).jsonObject["layers"]?.jsonArray ?: return null
        if (layers.size != symbol.layers.size) return null

        FloatArray(layers.size * FRAME_STRIDE).also { frames ->
            layers.forEachIndexed { index, item ->
                val layer = item.jsonObject
                val offset = index * FRAME_STRIDE
                frames[offset] = layer.number("scale") ?: return null
                frames[offset + 1] = layer.number("dx") ?: return null
                frames[offset + 2] = layer.number("dy") ?: return null
                frames[offset + 3] = layer.number("rotation") ?: return null
                frames[offset + 4] = layer.number("alpha") ?: return null
                frames[offset + 5] = layer.number("draw") ?: return null
                frames[offset + 6] = layer.nullableNumber("pivotX") ?: return null
                frames[offset + 7] = layer.nullableNumber("pivotY") ?: return null
                frames[offset + 8] = layer.number("variableAlpha") ?: return null
                frames[offset + 9] = layer.number("variableDraw") ?: return null

                val appearance = layer["appearance"]?.jsonObject
                if (appearance == null) {
                    for (field in 10 until FRAME_STRIDE) frames[offset + field] = Float.NaN
                } else {
                    frames[offset + 14] = 1f
                    val color = appearance["color"] ?: return null
                    if (color == JsonNull) {
                        for (field in 10..13) frames[offset + field] = Float.NaN
                    } else {
                        val channels = color.jsonArray
                        if (channels.size != 4) return null
                        channels.forEachIndexed { channel, value ->
                            frames[offset + 10 + channel] = value.jsonPrimitive.content.toFloatOrNull()
                                ?.takeIf(Float::isFinite) ?: return null
                        }
                    }
                    val stops = appearance["gradient"] ?: return null
                    if (stops == JsonNull) {
                        frames[offset + 15] = 0f
                    } else {
                        val pair = stops.jsonArray
                        if (pair.size != 2) return null
                        frames[offset + 15] = 1f
                        pair.forEachIndexed { stop, rgba ->
                            val channels = rgba.jsonArray
                            if (channels.size != 4) return null
                            channels.forEachIndexed { channel, value ->
                                frames[offset + 16 + stop * 4 + channel] = value.jsonPrimitive.content
                                    .toFloatOrNull()?.takeIf(Float::isFinite) ?: return null
                            }
                        }
                    }
                }
            }
        }
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

private const val FRAME_STRIDE = 24

private fun JsonObject.number(key: String): Float? = this[key]?.jsonPrimitive?.content
    ?.toFloatOrNull()?.takeIf { it.isFinite() }

private fun JsonObject.nullableNumber(key: String): Float? {
    val value = this[key] ?: return null
    return if (value == JsonNull) Float.NaN else value.jsonPrimitive.content.toFloatOrNull()
        ?.takeIf { it.isFinite() }
}

private fun SfRenderingMode.toRustString() = when (this) {
    SfRenderingMode.Monochrome -> "monochrome"
    SfRenderingMode.Hierarchical -> "hierarchical"
    SfRenderingMode.Palette -> "palette"
    SfRenderingMode.Multicolor -> "multicolor"
}

private fun Color.toRustRgba() = buildJsonArray {
    add(kotlinx.serialization.json.JsonPrimitive(red))
    add(kotlinx.serialization.json.JsonPrimitive(green))
    add(kotlinx.serialization.json.JsonPrimitive(blue))
    add(kotlinx.serialization.json.JsonPrimitive(alpha))
}

private fun Color.isRustRepresentable() = colorSpace == ColorSpaces.Srgb &&
    red.isFinite() && red in 0f..1f && green.isFinite() && green in 0f..1f &&
    blue.isFinite() && blue in 0f..1f && alpha.isFinite() && alpha in 0f..1f

private fun SfSymbolEffect.toRustJson(): JsonObject = buildJsonObject {
    fun scope(value: SfEffectScope) = when (value) {
        SfEffectScope.WholeSymbol -> "whole_symbol"
        SfEffectScope.ByLayer -> "by_layer"
    }
    when (this@toRustJson) {
        is SfSymbolEffect.Appear -> { put("type", "appear"); put("scope", scope(this@toRustJson.scope)) }
        is SfSymbolEffect.Disappear -> { put("type", "disappear"); put("scope", scope(this@toRustJson.scope)) }
        is SfSymbolEffect.Bounce -> {
            put("type", "bounce")
            put("direction", if (direction == SfBounceDirection.Up) "up" else "down")
            put("scope", scope(this@toRustJson.scope))
        }
        is SfSymbolEffect.Pulse -> { put("type", "pulse"); put("scope", scope(this@toRustJson.scope)) }
        is SfSymbolEffect.VariableColor -> {
            put("type", "variable_color")
            put("style", if (style == SfVariableColorStyle.Cumulative) "cumulative" else "iterative")
            put("reversing", reversing)
            put("hideInactiveLayers", hideInactiveLayers)
        }
        is SfSymbolEffect.Scale -> { put("type", "scale"); put("up", up); put("returns", returns) }
        is SfSymbolEffect.Wiggle -> {
            put("type", "wiggle")
            when (val value = direction) {
                null -> Unit
                is SfWiggleDirection.Custom -> put("direction", buildJsonObject { put("custom", value.degrees) })
                else -> put("direction", when (value) {
                    SfWiggleDirection.Left -> "left"
                    SfWiggleDirection.Right -> "right"
                    SfWiggleDirection.Up -> "up"
                    SfWiggleDirection.Down -> "down"
                    SfWiggleDirection.Forward -> "forward"
                    SfWiggleDirection.Backward -> "backward"
                    SfWiggleDirection.Clockwise -> "clockwise"
                    SfWiggleDirection.CounterClockwise -> "counter_clockwise"
                    is SfWiggleDirection.Custom -> error("handled above")
                })
            }
            put("continuous", continuous)
        }
        is SfSymbolEffect.Rotate -> {
            put("type", "rotate")
            direction?.let { put("direction", if (it == SfRotateDirection.Clockwise) "clockwise" else "counter_clockwise") }
            put("scope", scope(this@toRustJson.scope))
            put("continuous", continuous)
        }
        is SfSymbolEffect.Breathe -> { put("type", "breathe"); put("pulsing", pulsing) }
        is SfSymbolEffect.DrawOn -> {
            put("type", "draw_on"); put("scope", scope(this@toRustJson.scope)); put("individually", individually)
        }
        is SfSymbolEffect.DrawOff -> {
            put("type", "draw_off"); put("scope", scope(this@toRustJson.scope)); put("individually", individually)
        }
    }
}
