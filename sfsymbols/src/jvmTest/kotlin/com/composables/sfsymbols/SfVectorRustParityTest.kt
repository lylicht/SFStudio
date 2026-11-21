package com.composables.sfsymbols

import com.composables.sfsymbols.core.SfData
import com.composables.sfsymbols.core.SfGlyph
import com.composables.sfsymbols.core.SfNode
import com.composables.sfsymbols.core.SfScaleId
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Test

class SfVectorRustParityTest {
    @Test
    fun rustVectorDecoderMatchesEveryKotlinGlyphAndOpticalVariant() {
        val actual = SfData.all().joinToString(separator = "\n", postfix = "\n") { info ->
            val glyph = checkNotNull(SfData.rustGlyph(info)) { "Rust glyph JSON missing for ${info.name}" }
            assertEquals("medium ${info.name}", glyphHash(SfData.glyphFromKotlin(info)), glyphHash(glyph))
            if (info.name == "heart.fill") {
                assertEquals(glyphHash(glyph), glyphHash(SfData.glyph(info)))
            }
            val smallKotlin = SfData.scaleGlyphFromKotlin(info, SfScaleId.Small)
            val small = SfData.rustScaleGlyph(info, SfScaleId.Small)
            assertEquals("small ${info.name}", smallKotlin?.let(::glyphHash), small?.let(::glyphHash))
            val largeKotlin = SfData.scaleGlyphFromKotlin(info, SfScaleId.Large)
            val large = SfData.rustScaleGlyph(info, SfScaleId.Large)
            assertEquals("large ${info.name}", largeKotlin?.let(::glyphHash), large?.let(::glyphHash))
            "${info.name}|${glyphHash(glyph)}|${small?.let(::geometryHash) ?: "-"}|${large?.let(::geometryHash) ?: "-"}"
        }

        val expected = javaClass.getResourceAsStream("/sf-vector-fingerprints.txt")
            ?.bufferedReader()
            ?.use { it.readText() }
            ?: error("missing Rust vector parity fingerprints")
        assertEquals(expected.replace("\r\n", "\n"), actual)
    }

    @Test
    fun rustFlattenedAnimationLayersMatchKotlinForEverySymbol() {
        for (info in SfData.all()) {
            val glyph = SfData.glyph(info)
            val actual = glyph.animationLayers
            assertNotNull("Rust animation layers missing for ${info.name}", actual)
            val symbol = SfSymbol(info)
            val expected = symbol.layersFromKotlinData()
            val exposed = symbol.layers
            assertEquals("${info.name} layer count", expected.size, actual!!.size)
            assertEquals("${info.name} exposed layer count", actual.size, exposed.size)
            for ((index, layer) in actual.withIndex()) {
                val reference = expected[index]
                val publicLayer = exposed[index]
                val label = "${info.name} layer $index"
                assertArrayEquals("$label contours", reference.contours, layer.contours)
                assertEquals("$label hierarchy", reference.hierarchicalLevel, layer.hierarchical)
                assertEquals("$label erases", reference.erases, layer.erases)
                assertEquals("$label color", reference.colorName, SfData.colorName(layer.colorId))
                assertEquals("$label opacity", reference.opacity, layer.opacity, 0f)
                assertEquals("$label pulse", reference.canPulse, layer.canPulse)
                assertEquals("$label rotate", reference.canRotate, layer.canRotate)
                assertEquals("$label auto variable", reference.autoVariable, layer.autoVariable)
                assertEquals("$label variable draw", reference.variableDraw, layer.variableDraw)
                assertEquals("$label root U", reference.rootU, layer.rootU, 0f)
                assertEquals("$label root V", reference.rootV, layer.rootV, 0f)
                assertArrayEquals("$label ties", reference.ties, layer.ties, 0f)
                assertEquals("$label stroke length", reference.strokeLength, layer.strokeLength, 0.0002f)
                assertEquals("$label group", reference.group, layer.group)
                assertEquals("$label tags", reference.tags, layer.tags.map { SfData.tag(it) }.toSet())
                assertEquals("$label public stroke length", layer.strokeLength, publicLayer.strokeLength, 0f)
            }
        }
    }

    @Test
    fun rustGlyphDataCodecRejectsMalformedAndIncompletePayloads() {
        assertNull(parseRustGlyphData("{broken"))
        assertNull(parseRustGlyphData("""{"geometry":{},"tree":[]}"""))
    }

    @Test
    fun rustLayerCodecKeepsLegacyFallbackAndRejectsInvalidFlattenedLayers() {
        val legacy = checkNotNull(parseRustGlyphData(singleLayerGlyphJson()))
        assertNull(legacy.animationLayers)

        val layer = """[{"contours":[0],"hierarchical":0,"erases":false,"colorId":null,"opacity":1.0,"canPulse":false,"canRotate":false,"autoVariable":false,"variableDraw":false,"rootU":null,"rootV":null,"ties":[],"strokeLength":10.0,"group":0,"tags":[]}]"""
        val decoded = checkNotNull(parseRustGlyphData(singleLayerGlyphJson(layer)))
        assertEquals(1, decoded.animationLayers?.size)
        assertNull(parseRustGlyphData(singleLayerGlyphJson("[]")))
        assertNull(parseRustGlyphData(singleLayerGlyphJson(layer.replace("10.0", "-1.0"))))
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
                    val expectedBounds = symbol.boundsFromKotlinData(weight, scale)
                    append(name).append('|')
                        .append(weight.wght).append('|')
                        .append(scale.name.lowercase()).append('|')
                        .append(boundsHash(expectedBounds))
                        .append('\n')
                    val rust = checkNotNull(sfRustGlyphBounds(symbol, weight.wght, scale)) {
                        "Rust bounds missing for $name ${weight.name} ${scale.name}"
                    }
                    assertArrayEquals(
                        "$name ${weight.name} ${scale.name}",
                        floatArrayOf(expectedBounds.left, expectedBounds.top, expectedBounds.right, expectedBounds.bottom),
                        rust,
                        0f,
                    )
                    assertEquals(expectedBounds, symbol.bounds(weight, scale))
                }
            }
        }
        val expected = javaClass.getResourceAsStream("/sf-vector-bounds.txt")
            ?.bufferedReader()
            ?.use { it.readText() }
            ?: error("missing Rust vector bounds fixture")
        assertEquals(expected.replace("\r\n", "\n"), actual)
    }

    @Test
    fun rustPointInterpolationMatchesTheKotlinRendererAcrossWeightsAndScales() {
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
        for (name in names) {
            val symbol = SfSymbol.named(name) ?: error("unknown SF Symbol $name")
            for (weight in SfWeight.values()) for (scale in SfScale.values()) {
                val rust = sfRustGlyphPoints(symbol, weight.wght, scale)
                assertNotNull("Rust points missing for $name ${weight.name} ${scale.name}", rust)
                assertArrayEquals(
                    "$name ${weight.name} ${scale.name}",
                    symbol.pointsFromKotlinData(weight.wght, scale),
                    rust!!,
                    0f,
                )
                assertArrayEquals(
                    "The production renderer must use the Rust points when available",
                    rust,
                    symbol.points(weight.wght, scale),
                    0f,
                )
            }
        }
    }
}

private fun singleLayerGlyphJson(animationLayers: String? = null): String {
    val animationLayerField = animationLayers?.let { "\"animationLayers\":$it," }.orEmpty()
    return """
        {
          "geometry":{"ops":[0,1,3],"regular":[0,0,10,0],"light":[0,0,10,0],"heavy":[0,0,10,0]},
          "anchors":[],
          "tree":[{"isGroup":false,"canPulse":false,"canRotate":false,"clearBehind":false,"animateTogether":false,"variableDraw":false,"autoVariable":false,"monoErase":false,"shapeErase":false,"hierarchical":0,"colorId":null,"opacity":1.0,"rootU":null,"rootV":null,"ties":[],"contours":[0],"tagIds":[],"children":[]}],
          $animationLayerField
          "layersExact":false,"backToFront":true,"gapless":false,"defaultVariableRendering":0,"wiggle":0,"wiggleAngle":0,"rotation":0
        }
    """.trimIndent()
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
