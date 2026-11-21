package com.composables.sfsymbols

import androidx.compose.ui.geometry.Rect
import com.composables.sfsymbols.core.SfData
import com.composables.sfsymbols.core.SfScaleId
import kotlin.test.Test
import kotlin.test.assertContentEquals
import kotlin.test.assertEquals
import kotlin.test.assertNotNull

class SfWebRustRenderingTest {
    @Test
    fun rustWasmProvidesPointsAnimationAndDrawFieldLevels() {
        val info = SfData.all().first { it.name == "wifi" }
        val kotlinGlyph = SfData.glyphFromKotlin(info)
        val rustGlyph = assertNotNull(SfData.rustGlyph(info))
        assertContentEquals(kotlinGlyph.reg, rustGlyph.reg)
        assertContentEquals(kotlinGlyph.tree.first().contours, rustGlyph.tree.first().contours)
        val opticalInfo = SfData.all().first { it.smallBlock >= 0 }
        val kotlinSmall = assertNotNull(SfData.scaleGlyphFromKotlin(opticalInfo, SfScaleId.Small))
        val rustSmall = assertNotNull(SfData.rustScaleGlyph(opticalInfo, SfScaleId.Small))
        assertContentEquals(kotlinSmall.reg, rustSmall.reg)

        val symbol = SfSymbol.named("wifi") ?: error("missing fixture symbol")
        val points = assertNotNull(sfRustGlyphPoints(symbol, 400f, SfScale.Medium))
        assertEquals(symbol.pointsFromKotlinData(400f).toList(), points.toList())
        val expectedBounds = symbol.boundsFromKotlinData(SfWeight.Regular, SfScale.Medium)
        val rustBounds = assertNotNull(sfRustGlyphBounds(symbol, 400f, SfScale.Medium))
        assertEquals(
            listOf(expectedBounds.left, expectedBounds.top, expectedBounds.right, expectedBounds.bottom),
            rustBounds.toList(),
        )
        assertEquals(expectedBounds, symbol.bounds(SfWeight.Regular, SfScale.Medium))

        val frames = assertNotNull(sfRustAnimationFrames(symbol, SfSymbolEffect.Pulse(), 0.4f, false, null))
        assertEquals(symbol.layers.size * RUST_FRAME_STRIDE, frames.size)

        val mask = ByteArray(112 * 112).apply { this[0] = 1; this[1] = 1 }
        val levels = assertNotNull(sfRustDrawFieldLevels(mask, 0f, 0f, 1f, Rect(0f, 0f, 112f, 112f), FloatArray(0)))
        assertEquals(listOf(0, 47, 255), listOf(levels[0], levels[1], levels[2]).map { it.toInt() and 0xff })
    }
}
