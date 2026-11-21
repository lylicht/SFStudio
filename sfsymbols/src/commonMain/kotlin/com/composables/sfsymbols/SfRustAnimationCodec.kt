package com.composables.sfsymbols

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.colorspace.ColorSpaces
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.add
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put

internal const val RUST_FRAME_STRIDE = 24

internal fun buildRustAnimationRequest(
    symbol: SfSymbol,
    effect: SfSymbolEffect,
    time: Float,
    rtl: Boolean,
    variableValue: Float?,
    mode: SfRenderingMode,
    tint: Color,
    palette: List<Color>,
    gradient: Boolean,
): ByteArray {
    val includeAppearance = tint.isRustRepresentable() && palette.all { it.isRustRepresentable() }
    return buildJsonObject {
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
    }.toString().encodeToByteArray()
}

internal fun parseRustAnimationFrames(response: String, layerCount: Int): FloatArray? {
    val layers = Json.parseToJsonElement(response).jsonObject["layers"]?.jsonArray ?: return null
    if (layers.size != layerCount) return null

    return FloatArray(layers.size * RUST_FRAME_STRIDE).also { frames ->
        layers.forEachIndexed { index, item ->
            val layer = item.jsonObject
            val offset = index * RUST_FRAME_STRIDE
            frames[offset] = layer.rustNumber("scale") ?: return null
            frames[offset + 1] = layer.rustNumber("dx") ?: return null
            frames[offset + 2] = layer.rustNumber("dy") ?: return null
            frames[offset + 3] = layer.rustNumber("rotation") ?: return null
            frames[offset + 4] = layer.rustNumber("alpha") ?: return null
            frames[offset + 5] = layer.rustNumber("draw") ?: return null
            frames[offset + 6] = layer.rustNullableNumber("pivotX") ?: return null
            frames[offset + 7] = layer.rustNullableNumber("pivotY") ?: return null
            frames[offset + 8] = layer.rustNumber("variableAlpha") ?: return null
            frames[offset + 9] = layer.rustNumber("variableDraw") ?: return null

            val appearance = layer["appearance"]?.jsonObject
            if (appearance == null) {
                for (field in 10 until RUST_FRAME_STRIDE) frames[offset + field] = Float.NaN
            } else {
                frames[offset + 14] = 1f
                val color = appearance["color"] ?: return null
                if (color == JsonNull) {
                    for (field in 10..13) frames[offset + field] = Float.NaN
                } else {
                    val channels = color.jsonArray
                    if (channels.size != 4) return null
                    channels.forEachIndexed { channel, value ->
                        frames[offset + 10 + channel] = value.jsonPrimitive.content
                            .toFloatOrNull()?.takeIf { it.isFinite() } ?: return null
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
                                .toFloatOrNull()?.takeIf { it.isFinite() } ?: return null
                        }
                    }
                }
            }
        }
    }
}

internal fun JsonObject.rustNumber(key: String): Float? = this[key]?.jsonPrimitive?.content
    ?.toFloatOrNull()?.takeIf { it.isFinite() }

internal fun JsonObject.rustNullableNumber(key: String): Float? {
    val value = this[key] ?: return null
    return if (value == JsonNull) Float.NaN else value.jsonPrimitive.content
        .toFloatOrNull()?.takeIf { it.isFinite() }
}

internal fun SfRenderingMode.toRustString(): String = when (this) {
    SfRenderingMode.Monochrome -> "monochrome"
    SfRenderingMode.Hierarchical -> "hierarchical"
    SfRenderingMode.Palette -> "palette"
    SfRenderingMode.Multicolor -> "multicolor"
}

internal fun Color.toRustRgba(): JsonArray = buildJsonArray {
    add(kotlinx.serialization.json.JsonPrimitive(red))
    add(kotlinx.serialization.json.JsonPrimitive(green))
    add(kotlinx.serialization.json.JsonPrimitive(blue))
    add(kotlinx.serialization.json.JsonPrimitive(alpha))
}

internal fun Color.isRustRepresentable(): Boolean = colorSpace == ColorSpaces.Srgb &&
    red.isFinite() && red in 0f..1f && green.isFinite() && green in 0f..1f &&
    blue.isFinite() && blue in 0f..1f && alpha.isFinite() && alpha in 0f..1f

internal fun SfSymbolEffect.toRustJson(): JsonObject = buildJsonObject {
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
