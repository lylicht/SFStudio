package com.composables.sfsymbols

import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.graphics.Path
import java.io.File
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Test

class SfDrawFieldRustParityTest {
    @Test
    fun rustDrawFieldBridgeReturnsRevealLevels() {
        val mask = ByteArray(GRID * GRID)
        mask[0] = 1
        mask[1] = 1

        val levels = sfRustDrawFieldLevels(mask, 0f, 0f, 1f, Rect(0f, 0f, 112f, 112f), FloatArray(0))

        assertNotNull("Rust draw-field C ABI should be available in JVM tests", levels)
        val actual = checkNotNull(levels)
        assertEquals(0, actual[0].toInt() and 0xff)
        assertEquals(47, actual[1].toInt() and 0xff)
        assertEquals(255, actual[2].toInt() and 0xff)
    }

    @Test
    fun rustDrawFieldMatchesRealKotlinSymbolLayers() {
        val names = listOf(
            "arrow.left",
            "arrow.up.right",
            "bell.fill",
            "checkmark",
            "figure.run",
            "figure.walk",
            "wifi",
        )
        val actual = buildString {
            var fields = 0
            for (name in names) {
                val symbol = SfSymbol.named(name) ?: error("unknown SF Symbol $name")
                val contours = symbol.contourPaths(SfWeight.Regular, SfScale.Medium)
                val symbolBounds = symbol.bounds(SfWeight.Regular, SfScale.Medium)
                for ((index, layer) in symbol.layers.withIndex()) {
                    if (layer.ties.isEmpty()) continue
                    val path = Path().also { result ->
                        for (contour in layer.contours) {
                            if (contour in contours.indices) result.addPath(contours[contour])
                        }
                    }
                    val bounds = path.getBounds()
                    val pad = maxOf(bounds.width, bounds.height) * 0.02f + 1f
                    val originX = bounds.left - pad
                    val originY = bounds.top - pad
                    val cell = (maxOf(bounds.width, bounds.height) + 2f * pad) / GRID
                    var pixelLevels: ByteArray? = null
                    val field = buildDrawField(path, symbolBounds, layer.ties) {
                        pixelLevels = it
                    } ?: continue
                    append(name).append('|')
                        .append(index).append('|')
                        .append(originX).append('|')
                        .append(originY).append('|')
                        .append(cell).append('|')
                        .append(symbolBounds.left).append('|')
                        .append(symbolBounds.top).append('|')
                        .append(symbolBounds.right).append('|')
                        .append(symbolBounds.bottom).append('|')
                        .append(layer.ties.joinToString(",")).append('|')
                    for (level in checkNotNull(pixelLevels)) {
                        append(if (level.toInt() == -1) '.' else LEVEL_CODES[level.toInt() and 0xff])
                    }
                    append('\n')
                    fields++
                }
            }
            assertTrue("no sample symbols had drawable rail ties", fields > 0)
        }

        if (System.getenv("SFSTUDIO_RECORD_DRAW_FIXTURE") == "1") {
            File("src/jvmTest/resources/sf-draw-fingerprints.txt").writeText(actual)
            return
        }
        val expected = javaClass.getResourceAsStream("/sf-draw-fingerprints.txt")
            ?.bufferedReader()?.use { it.readText() }
            ?: error("missing Kotlin/Rust draw-field parity fixture")
        assertEquals(expected.replace("\r\n", "\n"), actual)
    }

    private companion object {
        const val GRID = 112
        const val LEVEL_CODES = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijkl"
    }
}
