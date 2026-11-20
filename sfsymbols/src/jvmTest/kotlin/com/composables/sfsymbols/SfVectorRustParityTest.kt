package com.composables.sfsymbols

import com.composables.sfsymbols.core.SfData
import com.composables.sfsymbols.core.SfGlyph
import com.composables.sfsymbols.core.SfNode
import com.composables.sfsymbols.core.SfScaleId
import org.junit.Assert.assertEquals
import org.junit.Test

class SfVectorRustParityTest {
    @Test
    fun rustVectorDecoderMatchesEveryKotlinGlyphAndOpticalVariant() {
        val actual = SfData.all().joinToString(separator = "\n", postfix = "\n") { info ->
            val glyph = SfData.glyph(info)
            val small = SfData.scaleGlyph(info, SfScaleId.Small)?.let(::geometryHash) ?: "-"
            val large = SfData.scaleGlyph(info, SfScaleId.Large)?.let(::geometryHash) ?: "-"
            "${info.name}|${glyphHash(glyph)}|$small|$large"
        }

        val expected = javaClass.getResourceAsStream("/sf-vector-fingerprints.txt")
            ?.bufferedReader()
            ?.use { it.readText() }
            ?: error("missing Rust vector parity fingerprints")
        assertEquals(expected.replace("\r\n", "\n"), actual)
    }

    @Test
    fun rustVisibleBoundsMatchKotlinAcrossWeightsAndOpticalScales() {
        val names = listOf(
            "0.circle",
            "arrow.up.right",
            "cloud.sun.rain.fill",
            "figure.run",
            "gearshape.2",
            "heart.fill",
            "person.crop.circle.badge.checkmark",
            "person.crop.circle.fill",
            "square.dashed",
            "wifi",
        )
        val actual = buildString {
            for (name in names) {
                val symbol = SfSymbol.named(name) ?: error("unknown SF Symbol $name")
                for (weight in SfWeight.values()) for (scale in SfScale.values()) {
                    append(name).append('|')
                        .append(weight.wght).append('|')
                        .append(scale.name.lowercase()).append('|')
                        .append(boundsHash(symbol.bounds(weight, scale)))
                        .append('\n')
                }
            }
        }
        val expected = javaClass.getResourceAsStream("/sf-vector-bounds.txt")
            ?.bufferedReader()
            ?.use { it.readText() }
            ?: error("missing Rust vector bounds fixture")
        assertEquals(expected.replace("\r\n", "\n"), actual)
    }
}

private class Fingerprint {
    private var value = -3750763034362895579L

    fun byte(byte: Int) {
        value = (value xor (byte.toLong() and 0xff)) * 1099511628211L
    }

    fun int(int: Int) {
        repeat(4) { byte(int ushr (it * 8)) }
    }

    fun bool(bool: Boolean) = byte(if (bool) 1 else 0)
    fun float(float: Float) = int(float.toBits())

    fun floats(values: FloatArray) {
        int(values.size)
        values.forEach(::float)
    }

    fun ints(values: IntArray) {
        int(values.size)
        values.forEach(::int)
    }

    fun hex() = value.toULong().toString(16).padStart(16, '0')
}

private fun geometryHash(glyph: SfGlyph): String = Fingerprint().apply {
    int(glyph.ops.size)
    glyph.ops.forEach { byte(it.toInt()) }
    floats(glyph.reg)
    floats(glyph.lo)
    floats(glyph.hi)
}.hex()

private fun boundsHash(bounds: androidx.compose.ui.geometry.Rect): String = Fingerprint().apply {
    float(bounds.left)
    float(bounds.top)
    float(bounds.right)
    float(bounds.bottom)
}.hex()

private fun glyphHash(glyph: SfGlyph): String = Fingerprint().apply {
    int(glyph.ops.size)
    glyph.ops.forEach { byte(it.toInt()) }
    floats(glyph.reg)
    floats(glyph.lo)
    floats(glyph.hi)
    bool(glyph.layersExact)
    bool(glyph.backToFront)
    bool(glyph.gapless)
    byte(glyph.defaultVariableRendering)
    byte(glyph.wiggle)
    int(glyph.wiggleAngle)
    byte(glyph.rotation)
    floats(glyph.anchors)
    int(glyph.tree.size)
    glyph.tree.forEach { node(it) }
}.hex()

private fun Fingerprint.node(node: SfNode) {
    bool(node.isGroup)
    bool(node.canPulse)
    bool(node.canRotate)
    bool(node.clearBehind)
    bool(node.animateTogether)
    bool(node.variableDraw)
    bool(node.autoVariable)
    bool(node.monoErase)
    bool(node.shapeErase)
    int(node.hierarchical)
    int(node.colorId)
    float(node.opacity)
    floats(node.ties)
    ints(node.contours)
    ints(node.tags)
    int(node.children.size)
    node.children.forEach { node(it) }
}
