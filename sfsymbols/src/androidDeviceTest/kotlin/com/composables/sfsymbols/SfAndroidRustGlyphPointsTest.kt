package com.composables.sfsymbols

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertArrayEquals
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class SfAndroidRustGlyphPointsTest {
    @Test
    fun rustPointsMatchTheKotlinRendererAcrossWeightsAndScales() {
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
                    "The Android renderer must use the Rust points when available",
                    rust,
                    symbol.points(weight.wght, scale),
                    0f,
                )
                val expectedBounds = symbol.boundsFromKotlinData(weight, scale)
                val rustBounds = sfRustGlyphBounds(symbol, weight.wght, scale)
                assertNotNull("Rust bounds missing for $name ${weight.name} ${scale.name}", rustBounds)
                assertArrayEquals(
                    "$name ${weight.name} ${scale.name}",
                    floatArrayOf(expectedBounds.left, expectedBounds.top, expectedBounds.right, expectedBounds.bottom),
                    rustBounds!!,
                    0f,
                )
                assertEquals(expectedBounds, symbol.bounds(weight, scale))
            }
        }
    }
}
