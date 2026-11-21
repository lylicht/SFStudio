@file:OptIn(kotlin.js.ExperimentalWasmJsInterop::class)

package com.composables.sfsymbols

import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Color
import com.composables.sfsymbols.core.SfGlyph
import com.composables.sfsymbols.core.SfScaleId
import com.composables.sfsymbols.resources.Res
import kotlin.js.js
import org.jetbrains.compose.resources.ExperimentalResourceApi

private var rustWasmLoadStarted = false

internal actual fun sfRustGlyphData(name: String, scale: SfScaleId): SfGlyph? = runCatching {
    if (!ensureRustWasm()) return null
    val scaleId = when (scale) {
        SfScaleId.Medium -> 0
        SfScaleId.Small -> 1
        SfScaleId.Large -> 2
    }
    parseRustGlyphData(rustGlyphDataJson(name, scaleId) ?: return null)
}.getOrNull()

internal actual fun sfRustGlyphPoints(symbol: SfSymbol, weight: Float, scale: SfScale): FloatArray? = runCatching {
    if (!ensureRustWasm()) return null
    val scaleId = when (scale) {
        SfScale.Medium -> 0
        SfScale.Small -> 1
        SfScale.Large -> 2
    }
    val response = rustGlyphFloatArrayJson("sfstudio_glyph_points", symbol.name, scaleId, weight) ?: return null
    parseRustGlyphPoints(response)
}.getOrNull()

internal actual fun sfRustGlyphBounds(symbol: SfSymbol, weight: Float, scale: SfScale): FloatArray? = runCatching {
    if (!ensureRustWasm()) return null
    val scaleId = when (scale) {
        SfScale.Medium -> 0
        SfScale.Small -> 1
        SfScale.Large -> 2
    }
    val response = rustGlyphFloatArrayJson("sfstudio_glyph_bounds", symbol.name, scaleId, weight) ?: return null
    parseRustGlyphPoints(response)?.takeIf {
        it.size == 4 && it.all(Float::isFinite) && it[0] <= it[2] && it[1] <= it[3]
    }
}.getOrNull()

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
): FloatArray? = runCatching {
    if (!ensureRustWasm()) return null
    val request = buildRustAnimationRequest(symbol, effect, time, rtl, variableValue, mode, tint, palette, gradient).decodeToString()
    parseRustAnimationFrames(invokeRustJson("sfstudio_animation_json", request) ?: return null, symbol.layers.size)
}.getOrNull()

internal actual fun sfRustDrawField(
    mask: ByteArray,
    originX: Float,
    originY: Float,
    cell: Float,
    symbolBounds: Rect,
    ties: FloatArray,
): SfRustDrawField? = runCatching {
    if (!ensureRustWasm()) return null
    val request = buildRustDrawFieldRequest(mask, originX, originY, cell, symbolBounds, ties).decodeToString()
    parseRustDrawField(invokeRustJson("sfstudio_draw_field_json", request) ?: return null, mask.size)
}.getOrNull()

@OptIn(ExperimentalResourceApi::class)
private fun ensureRustWasm(): Boolean {
    if (hasRustWasm()) return true
    if (!rustWasmLoadStarted) {
        rustWasmLoadStarted = true
        runCatching {
            val url = if (isNodeRuntime()) "" else Res.getUri("files/sfstudio_ffi.wasm")
            loadRustWasm(url) {
                SfSymbol.clearCachedBounds()
                SfRustWasmRuntime.revision.intValue++
            }
        }
    }
    return hasRustWasm()
}

private fun hasRustWasm(): Boolean = js("globalThis.__sfstudioRustWasm != null")

private fun isNodeRuntime(): Boolean = js("typeof process !== 'undefined' && process.versions != null && process.versions.node != null")

private fun loadRustWasm(url: String, onReady: () -> Unit): Unit = js(
    """
    {
        const host = globalThis;
        try {
            const builtin = host.process && host.process.getBuiltinModule;
            const fs = builtin
                ? builtin.call(host.process, "fs")
                : (typeof require !== "undefined" ? require("fs") : null);
            if (fs) {
                const path = builtin
                    ? builtin.call(host.process, "path")
                    : require("path");
                const base = typeof __dirname !== "undefined"
                    ? __dirname
                    : path.join(host.process.cwd(), "kotlin");
                const resource = url
                    ? (url.startsWith("file:") ? new URL(url) : url)
                    : path.join(base, "composeResources", "com.composables.sfsymbols.resources", "files", "sfstudio_ffi.wasm");
                host.__sfstudioRustWasm = new WebAssembly.Instance(new WebAssembly.Module(fs.readFileSync(resource))).exports;
                onReady();
                return;
            }
            fetch(url)
                .then(response => { if (!response.ok) throw new Error("Rust wasm resource unavailable"); return response.arrayBuffer(); })
                .then(bytes => WebAssembly.compile(bytes))
                .then(module => {
                    host.__sfstudioRustWasm = new WebAssembly.Instance(module).exports;
                    onReady();
                })
                .catch(() => { host.__sfstudioRustWasm = null; });
        } catch (_) {
            host.__sfstudioRustWasm = null;
        }
    }
    """,
)

private fun rustGlyphFloatArrayJson(
    functionName: String,
    symbolName: String,
    scaleId: Int,
    requestedWeight: Float,
): String? = js(
    """
    ((ffi, exportName, name, scale, weight) => {
        if (!ffi) return null;
        const inputBytes = new TextEncoder().encode(name);
        const input = ffi.sfstudio_alloc(inputBytes.length);
        if (!input) return null;
        let output = 0;
        let outputBytes = 0;
        try {
            new Uint8Array(ffi.memory.buffer, input, inputBytes.length).set(inputBytes);
            const needed = ffi[exportName](input, inputBytes.length, scale, weight, 0, 0);
            if (!Number.isSafeInteger(needed) || needed < 0 || needed > 0x1fffffff) return null;
            if (needed === 0) return "[]";
            outputBytes = needed * Float32Array.BYTES_PER_ELEMENT;
            output = ffi.sfstudio_alloc(outputBytes);
            if (!output) return null;
            const written = ffi[exportName](input, inputBytes.length, scale, weight, output, needed);
            if (written !== needed) return null;
            return JSON.stringify(Array.from(new Float32Array(ffi.memory.buffer, output, needed)));
        } catch (_) {
            return null;
        } finally {
            if (output) ffi.sfstudio_free(output, outputBytes);
            ffi.sfstudio_free(input, inputBytes.length);
        }
    })(globalThis.__sfstudioRustWasm, functionName, symbolName, scaleId, requestedWeight)
    """,
)

// shortcut: browser Wasm loads asynchronously; add cache invalidation when its loaded signal can refresh glyphs.
private fun rustGlyphDataJson(symbolName: String, scaleId: Int): String? = js(
    """
    ((ffi, name, scale) => {
        if (!ffi) return null;
        const inputBytes = new TextEncoder().encode(name);
        const input = ffi.sfstudio_alloc(inputBytes.length);
        if (!input) return null;
        let output = 0;
        let outputBytes = 0;
        try {
            new Uint8Array(ffi.memory.buffer, input, inputBytes.length).set(inputBytes);
            const needed = ffi.sfstudio_glyph_json(input, inputBytes.length, scale, 400, 0, 0);
            if (!Number.isSafeInteger(needed) || needed <= 0 || needed > 0x1fffffff) return null;
            outputBytes = needed;
            output = ffi.sfstudio_alloc(outputBytes);
            if (!output) return null;
            const written = ffi.sfstudio_glyph_json(input, inputBytes.length, scale, 400, output, outputBytes);
            if (written !== needed) return null;
            return new TextDecoder("utf-8", { fatal: true }).decode(new Uint8Array(ffi.memory.buffer, output, written));
        } catch (_) {
            return null;
        } finally {
            if (output) ffi.sfstudio_free(output, outputBytes);
            ffi.sfstudio_free(input, inputBytes.length);
        }
    })(globalThis.__sfstudioRustWasm, symbolName, scaleId)
    """,
)

private fun invokeRustJson(exportName: String, jsonRequest: String): String? = js(
    """
    ((ffi, functionName, request) => {
        if (!ffi) return null;
        const inputBytes = new TextEncoder().encode(request);
        const input = ffi.sfstudio_alloc(inputBytes.length);
        if (!input) return null;
        let output = 0;
        let outputBytes = 0;
        try {
            new Uint8Array(ffi.memory.buffer, input, inputBytes.length).set(inputBytes);
            const evaluate = ffi[functionName];
            const needed = evaluate(input, inputBytes.length, 0, 0);
            if (!Number.isSafeInteger(needed) || needed <= 0) return null;
            outputBytes = needed;
            output = ffi.sfstudio_alloc(outputBytes);
            if (!output) return null;
            const written = evaluate(input, inputBytes.length, output, outputBytes);
            if (written !== needed) return null;
            return new TextDecoder("utf-8", { fatal: true }).decode(new Uint8Array(ffi.memory.buffer, output, written));
        } catch (_) {
            return null;
        } finally {
            if (output) ffi.sfstudio_free(output, outputBytes);
            ffi.sfstudio_free(input, inputBytes.length);
        }
    })(globalThis.__sfstudioRustWasm, exportName, jsonRequest)
    """,
)
