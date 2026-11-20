package com.composables.sfsymbols

import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.colorspace.ColorSpaces
import com.sun.jna.Library
import com.sun.jna.Native
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.buildJsonArray
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive
import kotlinx.serialization.json.put
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import java.util.concurrent.atomic.AtomicBoolean

private interface NativeAnimation : Library {
    fun sfstudio_animation_json(input: ByteArray, inputLength: Long, output: ByteArray?, outputCapacity: Long): Long
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
): FloatArray? = RustAnimation.frames(symbol, effect, time, rtl, variableValue, mode, tint, palette, gradient)

private object RustAnimation {
    private val warned = AtomicBoolean()
    private val native by lazy { loadNative() }
    private val output = ThreadLocal.withInitial { ByteArray(16 * 1024) }

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
        val library = native ?: return null
        val includeAppearance = tint.isRustRepresentable() && palette.all { it.isRustRepresentable() }
        val request = buildJsonObject {
            put("symbol", symbol.name)
            put("time", time)
            put("rtl", rtl)
            put("effect", effect.toRustJson())
            if (variableValue != null) put("variableValue", variableValue)
            if (includeAppearance) {
                put("rendering", buildJsonObject {
                    put("mode", mode.toRustString())
                    put("tint", tint.toRustRgba())
                    put("palette", buildJsonArray { palette.forEach { add(it.toRustRgba()) } })
                    put("gradient", gradient)
                })
            }
        }.toString().toByteArray(Charsets.UTF_8)

        var buffer = output.get()
        var size = library.sfstudio_animation_json(request, request.size.toLong(), buffer, buffer.size.toLong())
        if (size < 0) {
            warn("Rust animation evaluator returned $size")
            return null
        }
        if (size > buffer.size) {
            if (size > Int.MAX_VALUE) return null
            buffer = ByteArray(size.toInt())
            output.set(buffer)
            size = library.sfstudio_animation_json(request, request.size.toLong(), buffer, buffer.size.toLong())
        }
        if (size < 0 || size > buffer.size) return null
        val layers = kotlinx.serialization.json.Json.parseToJsonElement(
            buffer.decodeToString(0, size.toInt()),
        ).jsonObject["layers"]?.jsonArray ?: return null
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
                                ?.takeIf { it.isFinite() } ?: return null
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
    } catch (error: Exception) {
        warn(error.message ?: error::class.simpleName.orEmpty())
        null
    } catch (error: LinkageError) {
        warn(error.message ?: error::class.simpleName.orEmpty())
        null
    }

    private fun JsonObject.number(key: String): Float? = this[key]?.jsonPrimitive?.content
        ?.toFloatOrNull()?.takeIf { it.isFinite() }

    private fun JsonObject.nullableNumber(key: String): Float? {
        val value = this[key] ?: return null
        return if (value == JsonNull) Float.NaN else value.jsonPrimitive.content
            .toFloatOrNull()?.takeIf { it.isFinite() }
    }

    private fun loadNative(): NativeAnimation? = try {
        val explicitPath = System.getProperty("sfstudio.rust.animation.library")
        if (!explicitPath.isNullOrBlank()) return Native.load(explicitPath, NativeAnimation::class.java)
        val osName = System.getProperty("os.name")
        val name = when {
            osName.startsWith("Windows", ignoreCase = true) -> "sfstudio_ffi.dll"
            osName.contains("mac", ignoreCase = true) -> "libsfstudio_ffi.dylib"
            else -> "libsfstudio_ffi.so"
        }
        val resource = javaClass.getResourceAsStream("/native/$name") ?: return null
        val extracted = Files.createTempFile("sfstudio-rust-animation-", ".${name.substringAfterLast('.')}")
        extracted.toFile().deleteOnExit()
        resource.use { Files.copy(it, extracted, StandardCopyOption.REPLACE_EXISTING) }
        Native.load(extracted.toAbsolutePath().toString(), NativeAnimation::class.java)
    } catch (error: Exception) {
        warn(error.message ?: error::class.simpleName.orEmpty())
        null
    } catch (error: LinkageError) {
        warn(error.message ?: error::class.simpleName.orEmpty())
        null
    }

    private fun warn(message: String) {
        if (warned.compareAndSet(false, true)) {
            System.err.println("Rust animation module unavailable; Kotlin animation fallback will be used: $message")
        }
    }
}

private const val FRAME_STRIDE = 24

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
