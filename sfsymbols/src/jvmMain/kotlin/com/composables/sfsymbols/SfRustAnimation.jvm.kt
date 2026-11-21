package com.composables.sfsymbols

import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color
import com.sun.jna.Library
import com.sun.jna.Native
import java.nio.file.Files
import java.nio.file.StandardCopyOption
import java.util.concurrent.atomic.AtomicBoolean

private interface NativeAnimation : Library {
    fun sfstudio_glyph_points(
        symbolName: ByteArray,
        symbolNameLength: Long,
        scale: Int,
        weight: Float,
        output: FloatArray?,
        outputCapacity: Long,
    ): Long
    fun sfstudio_animation_json(input: ByteArray, inputLength: Long, output: ByteArray?, outputCapacity: Long): Long
    fun sfstudio_draw_field_json(input: ByteArray, inputLength: Long, output: ByteArray?, outputCapacity: Long): Long
}

internal actual fun sfRustGlyphPoints(symbol: SfSymbol, weight: Float, scale: SfScale): FloatArray? =
    RustAnimation.glyphPoints(symbol, weight, scale)

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

internal actual fun sfRustDrawFieldLevels(
    mask: ByteArray,
    originX: Float,
    originY: Float,
    cell: Float,
    symbolBounds: Rect,
    ties: FloatArray,
): ByteArray? = RustAnimation.drawFieldLevels(mask, originX, originY, cell, symbolBounds, ties)

private object RustAnimation {
    private val warned = AtomicBoolean()
    private val native by lazy { loadNative() }
    private val output = ThreadLocal.withInitial { ByteArray(16 * 1024) }

    fun glyphPoints(symbol: SfSymbol, weight: Float, scale: SfScale): FloatArray? = try {
        val library = native ?: return null
        val name = symbol.name.toByteArray(Charsets.UTF_8)
        val scaleId = when (scale) {
            SfScale.Medium -> 0
            SfScale.Small -> 1
            SfScale.Large -> 2
        }
        val required = library.sfstudio_glyph_points(name, name.size.toLong(), scaleId, weight, null, 0)
        if (required < 0) {
            warn("Rust glyph interpolation returned $required")
            return null
        }
        if (required > Int.MAX_VALUE) return null
        val points = FloatArray(required.toInt())
        val written = library.sfstudio_glyph_points(
            name, name.size.toLong(), scaleId, weight, points, points.size.toLong(),
        )
        if (written != required || points.any { !it.isFinite() }) {
            warn("Rust glyph interpolation returned an invalid point array")
            return null
        }
        points
    } catch (error: Exception) {
        warn(error.message ?: error::class.simpleName.orEmpty())
        null
    } catch (error: LinkageError) {
        warn(error.message ?: error::class.simpleName.orEmpty())
        null
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
        val library = native ?: return null
        val request = buildRustAnimationRequest(symbol, effect, time, rtl, variableValue, mode, tint, palette, gradient)

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
        parseRustAnimationFrames(buffer.decodeToString(0, size.toInt()), symbol.layers.size)
            ?: return null
    } catch (error: Exception) {
        warn(error.message ?: error::class.simpleName.orEmpty())
        null
    } catch (error: LinkageError) {
        warn(error.message ?: error::class.simpleName.orEmpty())
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
        val library = native ?: return null
        val request = buildRustDrawFieldRequest(mask, originX, originY, cell, symbolBounds, ties)

        var buffer = output.get()
        var size = library.sfstudio_draw_field_json(request, request.size.toLong(), null, 0)
        if (size < 0) {
            warn("Rust draw-field evaluator returned $size")
            return null
        }
        if (size > Int.MAX_VALUE) return null
        if (size > buffer.size) {
            buffer = ByteArray(size.toInt())
            output.set(buffer)
        }
        val written = library.sfstudio_draw_field_json(request, request.size.toLong(), buffer, buffer.size.toLong())
        if (written != size) return null
        parseRustDrawFieldLevels(buffer.decodeToString(0, written.toInt()), mask.size) ?: return null
    } catch (error: Exception) {
        warn(error.message ?: error::class.simpleName.orEmpty())
        null
    } catch (error: LinkageError) {
        warn(error.message ?: error::class.simpleName.orEmpty())
        null
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
            System.err.println("Rust native module unavailable; Kotlin fallback will be used: $message")
        }
    }
}
