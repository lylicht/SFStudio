package com.composables.sfsymbols

import kotlin.test.Test
import kotlin.test.assertTrue

class SfIosRustGlyphPointsTest {
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
                val rust = checkNotNull(sfRustGlyphPoints(symbol, weight.wght, scale)) {
                    "Rust points missing for $name ${weight.name} ${scale.name}"
                }
                val expected = symbol.pointsFromKotlinData(weight.wght, scale)
                assertTrue(
                    expected.contentEquals(rust),
                    "Rust and Kotlin points differ for $name ${weight.name} ${scale.name}",
                )
                assertTrue(
                    rust.contentEquals(symbol.points(weight.wght, scale)),
                    "The iOS renderer must use Rust points when available",
                )
            }
        }
    }
}
