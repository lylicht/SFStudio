@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package com.composables.sfsymbols

import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color
import com.composables.sfsymbols.core.SfGlyph
import com.composables.sfsymbols.core.SfScaleId
import com.composables.sfsymbols.ffi.sfstudio_animation_json
import com.composables.sfsymbols.ffi.sfstudio_draw_field_json
import com.composables.sfsymbols.ffi.sfstudio_glyph_json
import com.composables.sfsymbols.ffi.sfstudio_glyph_bounds
import com.composables.sfsymbols.ffi.sfstudio_glyph_points
import kotlinx.cinterop.UByteVar
import kotlinx.cinterop.addressOf
import kotlinx.cinterop.convert
import kotlinx.cinterop.reinterpret
import kotlinx.cinterop.usePinned

internal actual fun sfRustGlyphData(name: String, scale: SfScaleId): SfGlyph? = try {
    val bytes = name.encodeToByteArray()
    if (bytes.isEmpty()) return null
    val scaleId = when (scale) {
        SfScaleId.Medium -> 0
        SfScaleId.Small -> 1
        SfScaleId.Large -> 2
    }
    bytes.usePinned { pinnedName ->
        val namePointer = pinnedName.addressOf(0).reinterpret<UByteVar>()
        val required = sfstudio_glyph_json(
            namePointer, bytes.size.convert(), scaleId, 400f, null, 0uL,
        )
        if (required <= 0L || required > Int.MAX_VALUE.toLong()) return null
        val response = ByteArray(required.toInt())
        val written = response.usePinned { pinnedResponse ->
            sfstudio_glyph_json(
                namePointer, bytes.size.convert(), scaleId, 400f,
                pinnedResponse.addressOf(0).reinterpret<UByteVar>(), response.size.convert(),
            )
        }
        if (written != required) return null
        parseRustGlyphData(response.decodeToString())
    }
} catch (_: Exception) {
    null
}

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
            val namePointer = pinnedName.addressOf(0).reinterpret<UByteVar>()
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

internal actual fun sfRustGlyphBounds(symbol: SfSymbol, weight: Float, scale: SfScale): FloatArray? {
    val name = symbol.name.encodeToByteArray()
    if (name.isEmpty()) return null
    val scaleId = when (scale) {
        SfScale.Medium -> 0
        SfScale.Small -> 1
        SfScale.Large -> 2
    }
    return try {
        name.usePinned { pinnedName ->
            val namePointer = pinnedName.addressOf(0).reinterpret<UByteVar>()
            val required = sfstudio_glyph_bounds(
                namePointer, name.size.convert(), scaleId, weight, null, 0uL,
            )
            if (required != 4L) return null
            val bounds = FloatArray(4)
            bounds.usePinned { pinnedBounds ->
                val written = sfstudio_glyph_bounds(
                    namePointer, name.size.convert(), scaleId, weight,
                    pinnedBounds.addressOf(0), bounds.size.convert(),
                )
                if (written != required) return null
            }
            bounds.takeIf { result -> result.all { it.isFinite() } && result[0] <= result[2] && result[1] <= result[3] }
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
): FloatArray? = RustAnimation.frames(symbol, effect, time, rtl, variableValue, mode, tint, palette, gradient)

private object RustAnimation {
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
        val request = buildRustAnimationRequest(symbol, effect, time, rtl, variableValue, mode, tint, palette, gradient)
        request.usePinned { pinnedRequest ->
            val requestPointer = pinnedRequest.addressOf(0).reinterpret<UByteVar>()
            val required = sfstudio_animation_json(requestPointer, request.size.convert(), null, 0uL)
            if (required <= 0L || required > Int.MAX_VALUE.toLong()) return null
            val response = ByteArray(required.toInt())
            val written = response.usePinned { pinnedResponse ->
                sfstudio_animation_json(
                    requestPointer,
                    request.size.convert(),
                    pinnedResponse.addressOf(0).reinterpret<UByteVar>(),
                    response.size.convert(),
                )
            }
            if (written != required) return null
            parseRustAnimationFrames(response.decodeToString(), symbol.layers.size)
        }
    } catch (_: Exception) {
        null
    }
}

internal actual fun sfRustDrawFieldLevels(
    mask: ByteArray,
    originX: Float,
    originY: Float,
    cell: Float,
    symbolBounds: Rect,
    ties: FloatArray,
): ByteArray? = try {
    val request = buildRustDrawFieldRequest(mask, originX, originY, cell, symbolBounds, ties)
    request.usePinned { pinnedRequest ->
        val requestPointer = pinnedRequest.addressOf(0).reinterpret<UByteVar>()
        val required = sfstudio_draw_field_json(requestPointer, request.size.convert(), null, 0uL)
        if (required <= 0L || required > Int.MAX_VALUE.toLong()) return null
        val response = ByteArray(required.toInt())
        val written = response.usePinned { pinnedResponse ->
            sfstudio_draw_field_json(
                requestPointer,
                request.size.convert(),
                pinnedResponse.addressOf(0).reinterpret<UByteVar>(),
                response.size.convert(),
            )
        }
        if (written != required) return null
        parseRustDrawFieldLevels(response.decodeToString(), mask.size)
    }
} catch (_: Exception) {
    null
}
