package com.composables.sfsymbols

import androidx.compose.ui.geometry.Rect
import kotlin.test.Test
import kotlin.test.assertEquals
import kotlin.test.assertNotNull

class SfWebRustRenderingTest {
    @Test
    fun rustWasmProvidesPointsAnimationAndDrawFieldLevels() {
        val symbol = SfSymbol.named("wifi") ?: error("missing fixture symbol")
        val points = assertNotNull(sfRustGlyphPoints(symbol, 400f, SfScale.Medium))
        assertEquals(symbol.pointsFromKotlinData(400f).toList(), points.toList())

        val frames = assertNotNull(sfRustAnimationFrames(symbol, SfSymbolEffect.Pulse(), 0.4f, false, null))
        assertEquals(symbol.layers.size * RUST_FRAME_STRIDE, frames.size)

        val mask = ByteArray(112 * 112).apply { this[0] = 1; this[1] = 1 }
        val levels = assertNotNull(sfRustDrawFieldLevels(mask, 0f, 0f, 1f, Rect(0f, 0f, 112f, 112f), FloatArray(0)))
        assertEquals(listOf(0, 47, 255), listOf(levels[0], levels[1], levels[2]).map { it.toInt() and 0xff })
    }
}
